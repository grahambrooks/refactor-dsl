# Refactor DSL

[![Build Status](https://github.com/grahambrooks/refactor-dsl/workflows/CI/badge.svg)](https://github.com/grahambrooks/refactor-dsl/actions)
[![Documentation](https://img.shields.io/badge/docs-mdbook-blue)](https://grahambrooks.github.io/refactor-dsl/)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Crates.io](https://img.shields.io/crates/v/refactor-dsl.svg)](https://crates.io/crates/refactor-dsl)

A domain-specific language for multi-language code refactoring with Git-aware matching.

## Features

- **Git-aware matching** - Filter repositories by branch, commit history, and working state
- **Flexible file matching** - Glob patterns, extensions, and content-based filtering
- **AST-powered queries** - Find and transform code using tree-sitter syntax patterns
- **LSP integration** - Semantic refactoring through Language Server Protocol
- **Multi-language support** - Rust, TypeScript, JavaScript, and Python out of the box

## Quick Start

```rust
use refactor_dsl::prelude::*;

Refactor::in_repo("./my-project")
    .matching(|m| m
        .git(|g| g.branch("main"))
        .files(|f| f.extension("rs").exclude("**/target/**")))
    .transform(|t| t
        .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO\")"))
    .dry_run()
    .apply()?;
```

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
refactor-dsl = "0.1"
```

Or install the CLI:

```bash
cargo install refactor-dsl
```

## Documentation

- [**Getting Started Guide**](https://grahambrooks.github.io/refactor-dsl/getting-started.html) - Installation and first steps
- [**User Guide**](https://grahambrooks.github.io/refactor-dsl/) - Full documentation

### Core Concepts

| Topic | Description |
|-------|-------------|
| [Matchers](https://grahambrooks.github.io/refactor-dsl/matchers/) | File, Git, and AST matching |
| [Transforms](https://grahambrooks.github.io/refactor-dsl/transforms/) | Text and AST transformations |
| [Languages](https://grahambrooks.github.io/refactor-dsl/languages.html) | Supported programming languages |

### Advanced Topics

| Topic | Description |
|-------|-------------|
| [LSP Integration](https://grahambrooks.github.io/refactor-dsl/lsp/) | Semantic refactoring with LSP |
| [Multi-Repository](https://grahambrooks.github.io/refactor-dsl/multi-repo.html) | Working across multiple repos |
| [Tree-sitter Queries](https://grahambrooks.github.io/refactor-dsl/tree-sitter-queries.html) | Writing AST patterns |

### Reference

- [CLI Reference](https://grahambrooks.github.io/refactor-dsl/cli.html)
- [API Reference](https://grahambrooks.github.io/refactor-dsl/api.html)

## Use Cases

Refactor DSL is ideal for:

- **Codebase migrations** - Update API usage patterns across many files
- **Style enforcement** - Apply consistent code patterns
- **Bulk refactoring** - Rename symbols, update imports, transform patterns
- **Multi-repo operations** - Apply the same changes across multiple repositories
- **Automated code cleanup** - Remove deprecated patterns, add missing annotations

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Refactor DSL                         │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐ │
│  │  Matchers   │  │ Transforms  │  │  LSP Client     │ │
│  │             │  │             │  │                 │ │
│  │ - Git       │  │ - Text      │  │ - Rename        │ │
│  │ - File      │  │ - AST       │  │ - References    │ │
│  │ - AST       │  │ - File      │  │ - Definition    │ │
│  └─────────────┘  └─────────────┘  └─────────────────┘ │
├─────────────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────────┐   │
│  │              Language Support                    │   │
│  │                                                  │   │
│  │  Rust  │  TypeScript  │  Python  │  (extensible) │   │
│  └─────────────────────────────────────────────────┘   │
├─────────────────────────────────────────────────────────┤
│  tree-sitter │ git2 │ walkdir │ globset │ regex        │
└─────────────────────────────────────────────────────────┘
```

## Contributing

Contributions are welcome! Please read our [Contributing Guide](CONTRIBUTING.md) and [Code of Conduct](CODE_OF_CONDUCT.md) before submitting a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
