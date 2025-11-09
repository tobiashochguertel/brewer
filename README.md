# Brewer 🍺

An extremly fast [homebrew](https://brew.sh/) (macOS & Linux package manager)
CLI wrapper with extra features.

This is a WIP, alpha, early-stage, etc. project. A lot is missing and bugs are
expected.

[![asciicast](https://asciinema.org/a/xHsicw0FhBi0ehVJsuWxvtkWH.svg)](https://asciinema.org/a/xHsicw0FhBi0ehVJsuWxvtkWH)

## Features

- 🚀 **Lightning Fast**: 360x faster with intelligent caching (0.5s vs 3+ minutes)
- 🔍 Fuzzy formulae/cask search with an embedded [skim] ([fzf] rust alternative)
- 📦 Smart package validation before uninstall (no more trial-and-error)
- 🎯 Locate which formulae provides the given binary (Ubuntu's `command-not-found` equivalent)
- ⚡ Much faster than `brew search` (uses [nucleo] crate for non-interactive fuzzy search)
- 📋 Show plan before installing / uninstall kegs
- 💾 Intelligent caching system with configurable TTL

## Install

```bash
git clone https://github.com/tobiashochguertel/brewer.git
cd brewer

# using cargo
cargo install --path brewer_term --force

# or using `just`
just
```

## Uninstall

```bash
# Remove the brewer binary
rm ~/.cargo/bin/brewer

# Or if installed to a different location, find it first
which brewer  # Shows installation location
rm $(which brewer)
```

**Note**: Due to the binary being named `brewer` while the package is `brewer_term`, 
`cargo uninstall brewer_term` won't work. You must manually remove the binary.

## Performance

Brewer uses intelligent caching to provide lightning-fast responses:

| Taps | First Run (builds cache) | Subsequent Runs (cached) | Speedup |
|------|-------------------------|-------------------------|---------|
| 2-5  | 5-10 seconds           | <0.1s                   | 50-100x |
| 10-20| 30-60 seconds          | <0.1s                   | 300-600x|
| 50+  | 2-5 minutes            | 0.5s                    | **360x**|

### Cache Configuration

Control cache behavior with environment variables:

```bash
# Set cache TTL (time-to-live) in hours (default: 24)
export BREWER_CACHE_TTL=48  # Cache valid for 48 hours

# Disable automatic cache expiration (cache never expires)
export BREWER_CACHE_NEVER_EXPIRE=1

# Force cache refresh
export BREWER_CACHE_TTL=0

# Custom executables data source (default: Homebrew official)
export BREWER_EXECUTABLES_URL="https://formulae.brew.sh/api/internal/executables.txt"

# Custom cache directory (default: platform-specific cache dir)
export BREWER_CACHE_DIR="$HOME/.brewer_cache"
```

**Examples:**
```bash
# Use cache for a week
BREWER_CACHE_TTL=168 brewer which fd

# Never expire cache (manual updates only)
BREWER_CACHE_NEVER_EXPIRE=1 brewer list

# Force fresh data (bypass cache)
BREWER_CACHE_TTL=0 brewer search python

# Use custom cache directory (useful for testing or multiple configurations)
BREWER_CACHE_DIR=/tmp/brewer_test brewer cache update
```

### First Run

The first time you run brewer (or after cache expires), it will build the cache:

```bash
$ brewer which fd
# Building cache... (this takes 2-3 minutes with many taps)
# Subsequent commands will be instant!
```

### Cache Management

```bash
# Check cache status
brewer cache status

# Manually update cache
brewer cache update

# Clear cache
brewer cache clear
```

**Note**: Executables data (used by `which` command) is cached alongside brew data and follows the same TTL configuration.

## Troubleshooting

### Slow Performance?

If brewer is slow:
1. **First run?** Cache is being built (one-time, 2-5 min)
2. **Many taps?** Consider reducing to 10-15 actively used taps
3. **Cache expired?** Set `BREWER_CACHE_TTL` to keep cache longer

### Broken Taps Error

If you see errors like:
```
==> EOF while parsing a value at line 1 column 0
==> brew info --eval-all failed
```

This is caused by broken Homebrew taps on your system. See [BROKEN_TAPS_ISSUE.md](BROKEN_TAPS_ISSUE.md) for detailed solutions.

**Quick fix:**
```bash
# Check for broken taps
brew info --eval-all --json=v2 2>&1 | grep "Error"

# Remove problematic taps
brew untap adoptopenjdk/openjdk
brew untap kaos/shell
# ... remove others showing errors
```

## Usage

```
Usage: brewer [OPTIONS] <COMMAND>

Commands:
  which      Locate the formulae which provides the given executable
  update     Update the local cache
  list       List installed formulae and casks
  info       Show information about formula or cask
  search     Search for formulae and casks
  paths      Show paths that brewer uses
  exists     Indicate if the given formula or cask exists by exit code
  install    Install the given formula or cask
  uninstall  Uninstall the given formula or cask
  help       Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  More output per occurrence
  -q, --quiet...    Less output per occurrence
  -h, --help        Print help
  -V, --version     Print version
```

[fzf]: https://github.com/junegunn/fzf
[nucleo]: https://github.com/helix-editor/nucleo
[skim]: https://github.com/lotabout/skim
