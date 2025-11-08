# Quick Fix: JSON Parsing Error

## Error Message
```
invalid type: null, expected a string at line 13933 column 22
```

## What This Means

A formula in one of your taps has a `null` value where a string is expected. This is a **Homebrew data issue**, not a brewer bug.

## Step 1: Find the Problematic Formula

```bash
# Save brew output to file (takes 2-3 minutes)
brew info --eval-all --json=v2 2>/dev/null > /tmp/brew_full.json

# Find the problematic area
sed -n '13920,13950p' /tmp/brew_full.json

# Or use python to find it
python3 << 'EOF'
import json

with open('/tmp/brew_full.json', 'r') as f:
    try:
        data = json.load(f)
        print("JSON is valid!")
    except json.JSONDecodeError as e:
        print(f"Error at line {e.lineno}, column {e.colno}")
        print(f"Message: {e.msg}")
        
        # Show context around the error
        f.seek(0)
        lines = f.readlines()
        start = max(0, e.lineno - 10)
        end = min(len(lines), e.lineno + 10)
        for i in range(start, end):
            marker = ">>> " if i == e.lineno - 1 else "    "
            print(f"{marker}{i+1:5d}: {lines[i]}", end='')
EOF
```

## Step 2: Identify the Formula/Cask

Once you find the problematic line, look for the `"name":` or `"token":` field nearby to identify which formula/cask is broken.

Example:
```json
{
  "name": "problematic-formula",  <-- This is the broken one
  "desc": null,  <-- This should be a string or omitted
  ...
}
```

## Step 3: Fix Options

### Option A: Remove the Tap

If the formula is from a third-party tap you don't need:
```bash
# Find which tap contains the formula
brew info problematic-formula 2>&1 | grep "From:"

# Remove that tap
brew untap some-user/some-tap
```

### Option B: Report the Issue

Report to the tap maintainer:
```bash
# Find the tap's GitHub repo
brew info problematic-formula 2>&1 | grep "From:"

# Open an issue on GitHub
# Include the error message and line number
```

### Option C: Temporary Workaround

Make Rust/Serde more lenient with null values. This is what we'll implement in brewer to handle such cases gracefully.

## Step 4: Verify Fix

```bash
# Test that brew output is now valid JSON
brew info --eval-all --json=v2 2>/dev/null | jq . > /dev/null && echo "✅ JSON is valid"

# Test brewer
time brewer which fd
```

## Common Null Value Fields

These fields often have null instead of proper values:
- `desc` - Should be string or omitted
- `homepage` - Should be string or omitted  
- `license` - Should be string or omitted
- `caveats` - Should be string or omitted

## Temporary Workaround in Brewer

While you fix the Homebrew issue, we can make brewer more resilient:

```rust
// Make fields optional to handle null values
#[derive(Deserialize)]
struct Formula {
    name: String,
    #[serde(default, deserialize_with = "null_to_option")]
    desc: Option<String>,
    // ... other fields
}

fn null_to_option<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer).unwrap_or(None))
}
```

This makes brewer continue working even with bad data from brew.

## Most Likely Culprits

Based on your taps, these are most likely to have issues:
1. Personal/small taps (fewer maintainers)
2. Experimental taps
3. Archived/unmaintained taps

Check these first:
```bash
# Check each tap for activity
for tap in $(brew tap); do
  echo "==> $tap"
  ls -la "/opt/homebrew/Library/Taps/${tap/\//-}/homebrew-${tap##*/}" 2>/dev/null | head -5
done
```

## Long-term Solution

Once we implement the cache system (see CACHE_IMPLEMENTATION_PLAN.md), brewer will:
1. Cache the valid data it can parse
2. Skip problematic formulas with warnings
3. Continue working with partial data
4. Update cache incrementally

This way, one broken formula won't break the entire system.

## If You're Stuck

If you can't identify the problem formula:

1. **Binary search approach**: Disable half your taps, test, repeat
```bash
# List all non-core taps
brew tap | grep -v "homebrew/core\|homebrew/cask" > /tmp/taps.txt

# Disable half
head -n 30 /tmp/taps.txt | xargs -I {} brew untap {}

# Test
brew info --eval-all --json=v2 > /dev/null && echo "Issue in disabled taps"

# If it works now, re-enable taps one by one to find the culprit
```

2. **Use only core taps temporarily**:
```bash
# Untap everything except core
brew tap | grep -v "homebrew/core\|homebrew/cask" | xargs -n1 brew untap

# Test brewer (should work now)
brewer which fd

# Re-add taps one by one as needed
```

3. **Wait for cache implementation**: The cache system will handle this gracefully.

## Questions?

- Check BROKEN_TAPS_ISSUE.md for general tap problems
- Check CACHE_IMPLEMENTATION_PLAN.md for the long-term solution
- Open an issue on the brewer GitHub with your specific error details
