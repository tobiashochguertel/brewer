# E2E Testing Strategy for Brewer

## Overview

End-to-end tests ensure the entire brewer application works correctly with real data.

## Test Data Setup

### Using Git LFS for Cache Data

Due to the large size of brew cache data (~58MB), we use Git LFS to store test fixtures.

```bash
# Install git-lfs
brew install git-lfs
git lfs install

# Track test cache files
git lfs track "tests/fixtures/*.db"
git lfs track "tests/fixtures/*.json"
```

### Test Fixtures Structure

```
tests/
├── fixtures/
│   ├── brew_cache_small.json    # 100 formulae + 50 casks
│   ├── brew_cache_medium.json   # 1000 formulae + 500 casks  
│   ├── brew_cache_full.json     # Full dump (git-lfs)
│   └── test_db/                 # Pre-built cache database
└── e2e/
    ├── cache_test.rs            # Cache command tests
    ├── search_test.rs           # Search performance tests
    ├── which_test.rs            # Which command tests
    └── uninstall_test.rs        # Uninstall validation tests
```

## Test Categories

### 1. Cache Lifecycle Tests
- Cache creation from scratch
- Cache update
- Cache expiration
- Cache clear

### 2. Performance Tests
- First run (cache build) timing
- Cached run timing (< 1s)
- ENV variable effects on caching

### 3. Functional Tests
- Which command with cache
- Search with cache
- List with cache
- Info with cache

### 4. Error Handling Tests
- Missing cache behavior
- Corrupted cache recovery
- Null value handling in JSON

## Running E2E Tests

```bash
# Run all e2e tests
cargo test --test '*' -- --test-threads=1

# Run specific test suite
cargo test --test cache_e2e

# Run with output
cargo test --test cache_e2e -- --nocapture

# Run with timing
cargo test --test cache_e2e -- --nocapture --test-threads=1 | tee test_results.txt
```

## Test Environment Setup

```bash
# Set test cache directory
export BREWER_TEST_CACHE_DIR=/tmp/brewer_test_cache

# Use fast TTL for testing
export BREWER_CACHE_TTL=0

# Clean test environment
rm -rf /tmp/brewer_test_cache
```

## CI/CD Integration

```yaml
# .github/workflows/e2e.yml
name: E2E Tests

on: [push, pull_request]

jobs:
  e2e:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v3
        with:
          lfs: true
      
      - name: Checkout LFS objects
        run: git lfs checkout
      
      - name: Install Homebrew
        run: /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run E2E Tests
        run: cargo test --test '*' -- --test-threads=1
```

## Test Data Generation

To generate test fixtures:

```bash
# Generate small test data
brew info --json=v2 $(brew list --formula | head -100) > tests/fixtures/brew_cache_small.json

# Generate medium test data  
brew info --json=v2 $(brew list --formula | head -1000) > tests/fixtures/brew_cache_medium.json

# Generate full dump (requires git-lfs)
brew info --eval-all --json=v2 > tests/fixtures/brew_cache_full.json
git lfs track tests/fixtures/brew_cache_full.json
git add tests/fixtures/brew_cache_full.json
git commit -m "Add full brew cache fixture via git-lfs"
```

## Performance Benchmarks

Expected timings (on system with 59 taps):

| Operation | Without Cache | With Cache | Target |
|-----------|--------------|------------|--------|
| which fd | 180s | 0.5s | <1s |
| search python | 180s | 0.3s | <1s |
| list | 180s | 0.4s | <1s |
| info helix | 180s | 0.5s | <1s |

## Test Coverage Goals

- ✅ Unit tests: 37 tests passing
- 🔄 Integration tests: In progress
- 🔄 E2E tests: In progress (this document)
- Target: 80%+ coverage

## Known Test Challenges

1. **Timing Variability**: Network and system load affect timing
2. **Tap Differences**: Different systems have different taps
3. **Brew Version**: brew API changes between versions
4. **Platform Differences**: macOS vs Linux behavior

## Solutions

1. Use relative timing (should be 10x faster, not exact ms)
2. Use controlled test fixtures
3. Pin brew command expectations to specific versions
4. Platform-specific test cases where needed

## Maintenance

- Update fixtures monthly or when brew format changes
- Review test failures in CI before merging
- Keep fixture sizes minimal (use small.json for most tests)
- Only use full.json for performance benchmarks
