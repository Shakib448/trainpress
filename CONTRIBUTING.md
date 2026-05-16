# Contributing to TrainPress

Thank you for your interest in contributing to TrainPress! This document provides guidelines and information for contributors.

## Project Structure

```
trainpress/
├── Cargo.toml              # Package configuration
├── README.md               # Main documentation
├── CONTRIBUTING.md         # This file
├── LICENSE                 # MIT License
│
├── src/                    # Library source code
│   ├── lib.rs             # Public API and exports
│   ├── app.rs             # Application builder
│   ├── router.rs          # Trie-based routing (matchit)
│   ├── handler.rs         # Handler type definitions
│   ├── middleware.rs      # Middleware trait and built-ins
│   ├── server.rs          # TCP listener and graceful shutdown
│   ├── response.rs        # IntoResponse implementations
│   ├── extract.rs         # Request extractors
│   └── error.rs           # Error types and HTTP mapping
│
├── examples/               # Example applications
│   ├── hello_world.rs     # Simple hello world server
│   ├── user_crud.rs       # Full CRUD API with state
│   └── middleware_example.rs  # Custom middleware demo
│
└── tests/                  # Integration tests
    └── integration_tests.rs   # Full application tests
```

## Development Setup

### Prerequisites

- **Rust 1.75+** (2024 edition)
- **Cargo**

### Clone and Build

```bash
git clone https://github.com/yourusername/trainpress
cd trainpress

# Build the library
cargo build

# Build examples
cargo build --examples

# Run tests
cargo test
```

## Running Examples

```bash
# Hello world example
cargo run --example hello_world

# Full CRUD API
cargo run --example user_crud

# Middleware demonstration
cargo run --example middleware_example
```

## Testing

### Run All Tests

```bash
cargo test
```

### Run Specific Test Suites

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# Run with output
cargo test -- --nocapture

# Run with logging
RUST_LOG=debug cargo test
```

### Test Coverage

- **Unit Tests** (48 tests): Test individual modules
  - Router: Path matching, parameters, wildcards
  - Extract: Path params, query strings, JSON, state
  - Error: Status codes, error responses
  - Response: Type conversions
  - Middleware: Composition and execution

- **Integration Tests** (17 tests): Test full application assembly
  - Route registration
  - Middleware stacking
  - State management
  - Builder pattern API

### Writing Tests

Add unit tests in the same file:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() {
        // Your test here
    }

    #[tokio::test]
    async fn test_async_something() {
        // Async test here
    }
}
```

Add integration tests in `tests/` directory.

## Code Style

### Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check
```

### Linting

```bash
# Run Clippy
cargo clippy

# Run Clippy with all features
cargo clippy --all-features
```

### Documentation

```bash
# Build documentation
cargo doc --no-deps --open

# Test documentation examples
cargo test --doc
```

## Contribution Guidelines

### Reporting Bugs

1. **Search existing issues** first
2. **Provide minimal reproduction** case
3. **Include version information**:
   - Rust version (`rustc --version`)
   - TrainPress version
   - Operating system
4. **Describe expected vs actual behavior**

### Suggesting Features

1. **Check if feature already requested**
2. **Explain the use case** clearly
3. **Consider impact** on existing API
4. **Propose implementation** approach if possible

### Submitting Pull Requests

1. **Fork the repository**
2. **Create a feature branch**:
   ```bash
   git checkout -b feature/your-feature-name
   ```

3. **Make your changes**:
   - Write clear, documented code
   - Add tests for new functionality
   - Ensure all tests pass
   - Run `cargo fmt` and `cargo clippy`

4. **Commit with clear messages**:
   ```bash
   git commit -m "Add feature: brief description"
   ```

5. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   ```

6. **Describe your changes** in the PR:
   - What problem does it solve?
   - How does it work?
   - Any breaking changes?
   - Related issues?

### Code Review Process

- Maintainers will review your PR
- Address feedback promptly
- Keep commits clean and organized
- Be respectful and constructive

## Development Tips

### Adding a New Feature

1. **Design first** - Consider API ergonomics
2. **Write tests** - TDD approach recommended
3. **Implement** - Keep it simple and clear
4. **Document** - Add examples and doc comments
5. **Test thoroughly** - Unit + integration tests

### Adding an Example

1. Create new file in `examples/` directory
2. Add clear comments explaining the example
3. Test the example works: `cargo run --example your_example`
4. Add entry to README examples section

### Improving Documentation

- Clear, concise language
- Code examples that compile
- Explain **why**, not just **what**
- Consider beginners' perspective

## Performance Considerations

TrainPress is built for performance. When contributing:

- **Avoid unnecessary allocations**
- **Use zero-copy when possible**
- **Consider async/await overhead**
- **Profile before optimizing**
- **Benchmark significant changes**

## Questions?

- Open an issue for questions
- Check existing issues and discussions
- Read the source code - it's designed to be readable!

## License

By contributing, you agree that your contributions will be licensed under the MIT License.

---

Thank you for contributing to TrainPress! 🦀🚀
