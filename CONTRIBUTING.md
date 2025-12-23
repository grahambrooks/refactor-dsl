# Contributing to Refactor DSL

Thank you for your interest in contributing to Refactor DSL! This document provides guidelines and information for contributors.

## Getting Started

### Prerequisites

- Rust 1.85 or later (edition 2024)
- Git

### Setting Up the Development Environment

1. Fork the repository
2. Clone your fork:
   ```bash
   git clone https://github.com/YOUR_USERNAME/refactor-dsl.git
   cd refactor-dsl
   ```
3. Build the project:
   ```bash
   cargo build
   ```
4. Run tests:
   ```bash
   cargo test
   ```

## How to Contribute

### Reporting Bugs

Before creating a bug report, please check the existing issues to avoid duplicates. When creating a bug report, include:

- A clear, descriptive title
- Steps to reproduce the issue
- Expected behavior vs actual behavior
- Your environment (OS, Rust version, etc.)
- Relevant code snippets or error messages

### Suggesting Features

Feature requests are welcome! Please provide:

- A clear description of the feature
- The use case it addresses
- Any implementation ideas you have

### Pull Requests

1. Create a new branch for your work:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. Make your changes, following the code style guidelines below

3. Add or update tests as needed

4. Ensure all tests pass:
   ```bash
   cargo test
   ```

5. Run clippy and fix any warnings:
   ```bash
   cargo clippy
   ```

6. Format your code:
   ```bash
   cargo fmt
   ```

7. Commit your changes with a clear message:
   ```bash
   git commit -m "Add feature: brief description"
   ```

8. Push to your fork and create a pull request

## Code Style Guidelines

### Rust Code

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` to catch common issues
- Write documentation comments for public APIs
- Keep functions focused and reasonably sized

### Commit Messages

- Use the present tense ("Add feature" not "Added feature")
- Use the imperative mood ("Move cursor to..." not "Moves cursor to...")
- Limit the first line to 72 characters
- Reference issues and PRs when relevant

### Documentation

- Update documentation when changing public APIs
- Add examples for new features
- Keep the README and docs in sync

## Project Structure

```
refactor-dsl/
├── src/
│   ├── lib.rs          # Library entry point
│   ├── bin/            # CLI binary
│   ├── matchers/       # File, Git, AST matchers
│   ├── transforms/     # Code transformations
│   └── lsp/            # LSP client integration
├── tests/              # Integration tests
├── examples/           # Usage examples
└── docs/               # mdBook documentation
```

## Testing

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

### Writing Tests

- Add unit tests in the same file as the code being tested
- Add integration tests in the `tests/` directory
- Test both success and failure cases
- Use descriptive test names

## Documentation

The documentation is built using mdBook. To work on documentation:

```bash
# Install mdBook
cargo install mdbook

# Build and serve locally
cd docs
mdbook serve
```

## Getting Help

If you need help or have questions:

- Open an issue for bugs or feature discussions
- Check existing issues and documentation first

## License

By contributing to Refactor DSL, you agree that your contributions will be licensed under the MIT License.
