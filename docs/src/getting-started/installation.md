# Installation

## As a Rust Library

Add Refactor DSL to your `Cargo.toml`:

```toml
[dependencies]
refactor = "0.1"
```

Then import the prelude in your code:

```rust
use refactor::prelude::*;
```

The prelude includes all commonly used types:

- `Refactor`, `MultiRepoRefactor`, `RefactorResult`
- `Matcher`, `FileMatcher`, `GitMatcher`, `AstMatcher`
- `Transform`, `TransformBuilder`, `TextTransform`, `AstTransform`
- `Language`, `LanguageRegistry`, `Rust`, `TypeScript`, `Python`, `Go`, `Java`, `CSharp`, `Ruby`
- `LspClient`, `LspRegistry`, `LspRename`, `LspInstaller`
- Refactoring: `ExtractFunction`, `InlineVariable`, `MoveToFile`, `ChangeSignature`, `SafeDelete`, `FindDeadCode`
- Discovery: `DependencyFilter`, `FrameworkFilter`, `MetricFilter`, `LanguageFilter`
- `RefactorError`, `Result`

## As a CLI Tool

### From Source

Clone and build the CLI:

```bash
git clone https://github.com/yourusername/refactor
cd refactor
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
| Java | jdtls | Eclipse JDT Language Server |
| C# | omnisharp | `dotnet tool install -g OmniSharp` |
| Ruby | solargraph | `gem install solargraph` |
| C/C++ | clangd | System package manager |

Or use auto-installation from the Mason registry (see [LSP Auto-Installation](../lsp/auto-install.md)).
