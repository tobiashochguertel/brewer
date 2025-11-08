# Testing Guide

## Overview

This project uses a modern, maintainable testing approach with comprehensive test coverage across all crates.

## Testing Strategy

### Unit Tests
- Located alongside source files using `#[cfg(test)]` modules
- Test individual functions and methods in isolation
- Focus on business logic, data structures, and pure functions

### Integration Tests
- Located in `tests/` directories within each crate
- Test interactions between modules and components
- Verify end-to-end workflows within a crate

## Test Organization

### brewer_core
- **Unit tests**: `src/lib_tests.rs`
  - Tests for `Brew` struct and builder
  - Tests for `Keg` enum and conversions
  - Tests for formula/cask model operations
  - Tests for `split_kegs` utility function

- **Integration tests**: 
  - `tests/integration_tests.rs` - Complete workflow scenarios and model creation
  - `tests/receipt_parsing_tests.rs` - INSTALL_RECEIPT.json parsing robustness
    - Tests handling of empty receipt files
    - Tests handling of corrupted JSON
    - Tests handling of missing receipt files
    - Tests mixed scenarios with valid and invalid receipts

### brewer_engine
- **Unit tests**: 
  - `src/lib_tests.rs` - Engine functionality
  - `src/store_tests.rs` - Store operations
  
- **Integration tests**: `tests/integration_tests.rs`
  - End-to-end engine workflows
  - Cache management scenarios
  - Store persistence tests

### brewer_term
- Currently focused on core library testing
- CLI testing can be added using integration tests with command execution

## Running Tests

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p brewer_core
cargo test -p brewer_engine
cargo test -p brewer_term

# Run with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name

# Run tests with verbose output
cargo test -- --test-threads=1 --nocapture
```

## Test Dependencies

### Production Dependencies
- Standard library testing framework (`#[test]`, `#[cfg(test)]`)

### Development Dependencies
- `tempfile`: Temporary directory and file creation for testing
- `mockito`: HTTP mocking for API tests (brewer_core)

## Best Practices

1. **Test Naming**: Use descriptive names starting with `test_`
   ```rust
   #[test]
   fn test_brew_default_values() { }
   ```

2. **Test Organization**: Group related tests in modules
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;
       // tests here
   }
   ```

3. **Assertions**: Use appropriate assertion macros
   - `assert!()` - boolean conditions
   - `assert_eq!()` - equality
   - `assert_ne!()` - inequality
   - `panic!()` - expected panics

4. **Test Isolation**: Each test should be independent
   - Use temporary directories for file operations
   - Clean up resources in tests
   - Avoid shared mutable state

5. **Test Data**: Create helper functions for common test data
   ```rust
   fn create_test_formula() -> Formula { }
   fn create_test_store() -> (Store, TempDir) { }
   ```

## Coverage Goals

- **brewer_core**: >80% coverage
  - Core data structures: 100%
  - Utility functions: 100%
  - Brew operations: Focus on testable logic
  - Error handling: Comprehensive coverage for file parsing failures

- **brewer_engine**: >80% coverage
  - Store operations: 100%
  - Cache management: 100%
  - Engine workflows: Focus on business logic

- **brewer_term**: >60% coverage
  - Configuration: High coverage
  - CLI utilities: Focus on testable components

## Continuous Integration

Tests should be run in CI/CD pipelines:
```yaml
# Example for GitHub Actions
- name: Run tests
  run: cargo test --all
```

## Future Improvements

1. **Property-based testing**: Add `proptest` or `quickcheck` for fuzz testing
2. **Benchmark tests**: Add criterion benchmarks for performance-critical code
3. **Mock objects**: Enhanced mocking for external dependencies
4. **CLI testing**: Add end-to-end CLI tests using `assert_cmd`
5. **Code coverage**: Integrate with tools like `tarpaulin` or `cargo-llvm-cov`

## Adding New Tests

When adding new functionality:

1. Write tests first (TDD approach recommended)
2. Include both positive and negative test cases
3. Test edge cases and error conditions
4. Update this document if adding new test patterns
5. Ensure tests are deterministic and don't depend on external state

## Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_basic_functionality() {
        // Arrange
        let input = setup_test_data();
        
        // Act
        let result = function_under_test(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
    
    #[test]
    #[should_panic(expected = "error message")]
    fn test_error_condition() {
        // Test code that should panic
    }
}
```

## Troubleshooting

### Common Issues

1. **Tests fail due to file permissions**: Use `tempfile::TempDir` for temporary directories
2. **Tests fail intermittently**: Check for race conditions or shared state
3. **Tests are slow**: Consider using `cargo test --release` or parallelization
4. **Database locked errors**: Ensure proper cleanup of database connections

### Getting Help

- Check test output with `--nocapture` flag
- Use `RUST_LOG=debug cargo test` for logging
- Review individual test documentation
- Consult the Rust testing guide: https://doc.rust-lang.org/book/ch11-00-testing.html
