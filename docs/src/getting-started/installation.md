# Installation

## As a Rust Library

Add Refactor DSL to your `Cargo.toml`:

```toml
[dependencies]
refactor-dsl = "0.1"
```

Then import the prelude in your code:

```rust
use refactor_dsl::prelude::*;
```

The prelude includes all commonly used types:

- `Refactor`, `MultiRepoRefactor`, `RefactorResult`
- `Matcher`, `FileMatcher`, `GitMatcher`, `AstMatcher`
- `Transform`, `TransformBuilder`, `TextTransform`, `AstTransform`
- `Language`, `LanguageRegistry`, `Rust`, `TypeScript`, `Python`
- `LspClient`, `LspRegistry`, `LspRename`, `LspInstaller`
- `RefactorError`, `Result`

## As a CLI Tool

### From Source

Clone and build the CLI:

```bash
git clone https://github.com/yourusername/refactor-dsl
cd refactor-dsl
cargo install --path .
```

### Verify Installation

```bash
refactor --version
refactor languages  # List supported languages
```

## Dependencies

Refactor DSL uses these key dependencies:

| Dependency | Purpose |
|------------|---------|
| `tree-sitter` | Multi-language parsing |
| `git2` | Git repository operations |
| `walkdir` | File system traversal |
| `globset` | Glob pattern matching |
| `regex` | Regular expressions |
| `lsp-types` | LSP protocol types |

### Optional LSP Servers

For semantic refactoring (rename, find references), you'll need language servers:

| Language | Server | Install |
|----------|--------|---------|
| Rust | rust-analyzer | `rustup component add rust-analyzer` |
| TypeScript | typescript-language-server | `npm i -g typescript-language-server` |
| Python | pyright | `npm i -g pyright` |
| Go | gopls | `go install golang.org/x/tools/gopls@latest` |
| C/C++ | clangd | System package manager |

Or use auto-installation from the Mason registry (see [LSP Auto-Installation](../lsp/auto-install.md)).
