# Contributing to apalis-cron

Thank you for your interest in contributing to apalis-cron! We welcome contributions from everyone.

## Getting Started

1. Fork the repository on GitHub
2. Clone your fork locally
3. Create a new branch for your feature or bug fix
4. Make your changes
5. Run the tests to ensure everything works
6. Submit a pull request

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Git

### Building the Project

```bash
git clone https://github.com/apalis-dev/apalis-cron.git
cd apalis-cron
cargo build
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with all features
cargo test --all-features

# Run tests for specific features
cargo test --features serde
cargo test --features english
```

### Code Quality

Before submitting a pull request, please ensure:

```bash
# Format your code
cargo fmt

# Run clippy for linting
cargo clippy --all-targets --all-features -- -D warnings

# Build documentation
cargo doc --all-features --no-deps
```

## Contribution Guidelines

### Code Style

- Follow the standard Rust formatting (run `cargo fmt`)
- Use descriptive variable and function names
- Add documentation for public APIs
- Include tests for new functionality

### Commit Messages

- Use clear and descriptive commit messages
- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters or less

### Pull Requests

- Fill out the pull request template completely
- Include tests for new features or bug fixes
- Update documentation if necessary
- Ensure all CI checks pass

### Issues

When reporting bugs or requesting features:

- Use the appropriate issue template
- Provide as much detail as possible
- Include code examples when relevant
- Check for existing issues before creating new ones

## Feature Development

When adding new features:

1. **Discuss first**: For significant changes, open an issue to discuss the approach
2. **Start small**: Break large features into smaller, reviewable chunks
3. **Test thoroughly**: Include unit tests and integration tests
4. **Document**: Update README.md and add doc comments
5. **Maintain compatibility**: Avoid breaking changes when possible

## Testing

We use several types of tests:

- **Unit tests**: Test individual functions and modules
- **Integration tests**: Test feature interactions
- **Doc tests**: Ensure code examples in documentation work
- **Feature tests**: Test different feature combinations

## Documentation

- Use `///` for public API documentation
- Include examples in documentation when helpful
- Update the README.md for user-facing changes
- Add entries to CHANGELOG.md

## Release Process

Releases are handled by maintainers:

1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create a git tag
4. GitHub Actions automatically publishes to crates.io

## Getting Help

- Open an issue for questions about contributing
- Check existing issues and pull requests
- Read the documentation at [docs.rs](https://docs.rs/apalis-cron)

## Code of Conduct

This project follows the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct).

## License

By contributing to apalis-cron, you agree that your contributions will be licensed under the MIT License.
