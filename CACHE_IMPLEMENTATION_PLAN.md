# Cache Implementation Plan

## Current Issue

**Problem**: `brew info --eval-all --json=v2` takes 3+ minutes with 59 taps
**Impact**: Every brewer command is extremely slow
**JSON Error**: `invalid type: null, expected a string at line 13933` - A formula has null instead of string

## Immediate Fix for JSON Error

The error is in Homebrew's formula data. To find which formula:

```bash
# Save output to file (this takes 3 minutes)
brew info --eval-all --json=v2 2>/dev/null > /tmp/brew_full.json

# Find the problematic line
sed -n '13930,13940p' /tmp/brew_full.json

# Or use jq to validate and find the issue
jq . /tmp/brew_full.json 2>&1 | grep -A 5 -B 5 "line 13933"
```

**Workaround**: The issue is likely in a specific formula. Once we have the cache system, we can handle null values gracefully.

## Cache System Design

### Architecture

```
brewer_core/
  └── cache/
      ├── mod.rs          # Cache trait and types
      ├── disk.rs         # Disk-based cache using cacache
      └── memory.rs       # Optional in-memory layer

brewer_engine/
  └── lib.rs             # Use cache instead of direct brew calls

brewer_term/
  └── cli.rs
      └── cache/
          ├── mod.rs      # Cache command module
          ├── status.rs   # Show cache status
          ├── update.rs   # Update cache
          ├── clear.rs    # Clear cache
          └── warmup.rs   # Pre-warm cache
```

### Crate Choice: `cacache`

**Why cacache**:
- Battle-tested (used by npm, cargo)
- Content-addressable storage
- Automatic integrity checking
- Concurrent access safe
- Size limits and expiration
- Fast lookups

**Add to Cargo.toml**:
```toml
[dependencies]
cacache = "13.0"
serde_json = "1.0"
```

### Cache Strategy

1. **Cache Key**: `brew-info-eval-all-v1`
2. **Cache Location**: `~/.cache/brewer/` (XDG compliant)
3. **Cache Duration**: 24 hours default (configurable)
4. **Cache Update**: Background process or explicit command

### Commands

```bash
# Show cache status
brewer cache status
  Age: 2 hours ago
  Size: 45 MB
  Entries: 8,542 formulae, 4,231 casks
  Valid: Yes
  Expires: in 22 hours

# Update cache (foreground)
brewer cache update
  Fetching from brew... (this will take 3 minutes)
  [████████████████████] 100%
  Updated successfully!

# Update cache (background)
brewer cache update --background
  Cache update started in background
  PID: 12345
  Check status with: brewer cache status

# Clear cache
brewer cache clear
  Cache cleared successfully

# Warm up cache (run on startup/cron)
brewer cache warmup
  Checking cache...
  Cache age: 25 hours (expired)
  Updating in background...
```

## Implementation Steps

### Phase 1: Basic Caching (Priority 1)

```rust
// brewer_core/src/cache.rs
use cacache;
use serde::{Deserialize, Serialize};

pub struct Cache {
    cache_dir: PathBuf,
}

impl Cache {
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| anyhow!("No cache dir"))?
            .join("brewer");
        
        fs::create_dir_all(&cache_dir)?;
        Ok(Cache { cache_dir })
    }
    
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match cacache::read(&self.cache_dir, key).await {
            Ok(data) => Ok(Some(data)),
            Err(cacache::Error::EntryNotFound(..)) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub async fn set(&self, key: &str, data: Vec<u8>) -> Result<()> {
        cacache::write(&self.cache_dir, key, data).await?;
        Ok(())
    }
}
```

### Phase 2: Smart Cache in Engine

```rust
// brewer_engine/src/lib.rs
impl Engine {
    pub fn cache_or_latest(&mut self) -> anyhow::Result<State> {
        // Try cache first
        if let Ok(Some(cached)) = self.cache.get("brew-state") {
            if !self.cache_expired()? {
                return Ok(cached);
            }
        }
        
        // Cache miss or expired - fetch fresh data
        eprintln!("📦 Updating cache... (this may take a few minutes with {} taps)", 
                  self.count_taps()?);
        
        let state = self.fetch_latest()?;
        self.cache.set("brew-state", state.clone())?;
        Ok(state)
    }
}
```

### Phase 3: CLI Commands

```rust
// brewer_term/src/cli.rs
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands
    Cache(cache::Cache),
}

// brewer_term/src/cli/cache/mod.rs
#[derive(Args)]
pub struct Cache {
    #[command(subcommand)]
    pub command: CacheCommands,
}

#[derive(Subcommand)]
pub enum CacheCommands {
    /// Show cache status
    Status,
    /// Update cache
    Update(Update),
    /// Clear cache
    Clear,
    /// Warm up cache
    Warmup,
}
```

## Handling JSON Parsing Errors

```rust
// Gracefully handle null values in JSON
#[derive(Deserialize)]
struct Formula {
    name: String,
    #[serde(default)]
    desc: Option<String>,  // Allow null
    #[serde(default)]
    homepage: Option<String>,  // Allow null
    // ... other fields with proper Option<T>
}

// Or use serde_json::Value for unknown structure
let result: serde_json::Value = serde_json::from_slice(&output)?;
// Then manually extract what we need
```

## Automation Scripts

### Cron Job (Update cache nightly)

```bash
#!/bin/bash
# ~/.local/bin/brewer-cache-update.sh

# Update cache in background
/Users/YOUR_USERNAME/.cargo/bin/brewer cache update --quiet

# Log result
echo "$(date): Cache updated" >> ~/.cache/brewer/update.log
```

Add to crontab:
```bash
# Update brewer cache at 3 AM daily
0 3 * * * ~/.local/bin/brewer-cache-update.sh
```

### Tmux Session (Update during idle)

```bash
#!/bin/bash
# Create detached tmux session for cache update

tmux new-session -d -s brewer-cache "brewer cache update; read"
echo "Cache update started in tmux session 'brewer-cache'"
echo "Attach with: tmux attach -t brewer-cache"
```

### LaunchAgent (macOS)

```xml
<!-- ~/Library/LaunchAgents/com.brewer.cache.plist -->
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>com.brewer.cache</string>
    <key>ProgramArguments</key>
    <array>
        <string>/Users/YOUR_USERNAME/.cargo/bin/brewer</string>
        <string>cache</string>
        <string>update</string>
        <string>--quiet</string>
    </array>
    <key>StartCalendarInterval</key>
    <dict>
        <key>Hour</key>
        <integer>3</integer>
        <key>Minute</key>
        <integer>0</integer>
    </dict>
    <key>StandardOutPath</key>
    <string>/tmp/brewer-cache.log</string>
    <key>StandardErrorPath</key>
    <string>/tmp/brewer-cache.err</string>
</dict>
</plist>
```

Load with:
```bash
launchctl load ~/Library/LaunchAgents/com.brewer.cache.plist
```

## README Section

```markdown
## Cache Management

Brewer caches Homebrew package information to provide fast lookups. With many taps (50+), 
the initial cache build can take 2-3 minutes, but subsequent commands are instant.

### Manual Cache Management

```bash
# Check cache status
brewer cache status

# Update cache manually
brewer cache update

# Clear cache
brewer cache clear
```

### Automatic Cache Updates

#### Using Cron (Linux/macOS)

```bash
# Edit crontab
crontab -e

# Add this line to update cache at 3 AM daily
0 3 * * * ~/.cargo/bin/brewer cache update --quiet
```

#### Using LaunchAgent (macOS)

```bash
# Download the plist file
curl -o ~/Library/LaunchAgents/com.brewer.cache.plist \
  https://raw.githubusercontent.com/metafates/brewer/main/scripts/com.brewer.cache.plist

# Update YOUR_USERNAME in the file
sed -i '' 's/YOUR_USERNAME/'$(whoami)'/g' ~/Library/LaunchAgents/com.brewer.cache.plist

# Load the agent
launchctl load ~/Library/LaunchAgents/com.brewer.cache.plist
```

#### Using Tmux (Manual Background Update)

```bash
# Start cache update in background tmux session
tmux new-session -d -s brewer-cache "brewer cache update; read"

# Detach and continue working
# Cache updates in background

# Check progress (optional)
tmux attach -t brewer-cache
```

### Cache Location

- Linux: `~/.cache/brewer/`
- macOS: `~/Library/Caches/brewer/`

### Performance

| Taps | First Run | Cached |
|------|-----------|--------|
| 2-5  | 5-10s     | <0.1s  |
| 10-20| 30-60s    | <0.1s  |
| 50+  | 2-5min    | <0.1s  |
```

## Migration Path

1. **Phase 1** (Now): Implement basic cache with cacache
2. **Phase 2**: Add cache commands (status, update, clear)
3. **Phase 3**: Add background update mechanism
4. **Phase 4**: Add automation scripts
5. **Phase 5**: Add cache integrity checking and repair

## Error Handling

```rust
// Handle various cache errors gracefully
match cache.get("brew-state") {
    Ok(Some(data)) => Ok(data),
    Ok(None) => {
        // Cache miss - fetch fresh
        fetch_and_cache()
    }
    Err(e) if e.is_corruption() => {
        log::warn!("Cache corrupted, rebuilding");
        cache.clear()?;
        fetch_and_cache()
    }
    Err(e) => {
        log::error!("Cache error: {}, falling back to direct fetch", e);
        fetch_without_cache()
    }
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_cache_read_write() {
        let cache = Cache::new().unwrap();
        let data = b"test data";
        cache.set("test-key", data).await.unwrap();
        assert_eq!(cache.get("test-key").await.unwrap(), Some(data));
    }
    
    #[test]
    fn test_cache_expiration() {
        // Test cache TTL
    }
    
    #[test]
    fn test_cache_corruption_recovery() {
        // Test recovery from corrupted cache
    }
}
```

## Future Enhancements

1. **Incremental Updates**: Only fetch changed formulas
2. **Compression**: Compress cached data (cacache supports this)
3. **Multiple Cache Levels**: Memory → Disk → Network
4. **Cache Sharing**: Share cache between users on same machine
5. **Pre-warming**: Warm cache on brew update hook

## Estimated Timeline

- **Week 1**: Basic cacache integration
- **Week 2**: Cache commands implementation
- **Week 3**: Automation scripts and documentation
- **Week 4**: Testing and refinement

## Success Metrics

- Cache hit rate: >95%
- Command response time with cache: <100ms
- Cache update time: Acceptable (2-5 min for 59 taps)
- Cache size: <100MB for typical setup
