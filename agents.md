# Development Guide for refactor-dsl

This document provides guidance for AI agents and developers working on the refactor-dsl codebase.

## Project Overview

refactor-dsl is a Rust library and CLI for multi-language code refactoring with:
- **Git-aware matching** - Filter repositories by branch, commits, file presence
- **File matching** - Glob patterns, extensions, content search
- **AST matching** - Tree-sitter queries for structural code matching
- **Transforms** - Text patterns, AST-aware rewrites, file operations
- **LSP integration** - Language Server Protocol support for semantic refactoring

## Architecture

```
src/
├── lib.rs              # Public API, prelude exports
├── error.rs            # Error types (RefactorError, Result)
├── refactor.rs         # Main Refactor builder, orchestration
├── diff.rs             # Diff generation (unified, colorized)
├── lang/               # Language abstraction
│   ├── mod.rs          # Language trait, LanguageRegistry
│   ├── rust.rs         # Rust support
│   ├── typescript.rs   # TypeScript/JS support
│   └── python.rs       # Python support
├── matcher/            # Matching predicates
│   ├── mod.rs          # Matcher builder combining git/file/ast
│   ├── git.rs          # Git repository predicates
│   ├── file.rs         # File system predicates
│   └── ast.rs          # AST queries via tree-sitter
├── transform/          # Code transformations
│   ├── mod.rs          # Transform trait, TransformBuilder
│   ├── text.rs         # Regex/literal text transforms
│   ├── ast.rs          # AST-aware transforms
│   └── file.rs         # File operations (move, rename, delete)
├── lsp/                # Language Server Protocol integration
│   ├── mod.rs          # LspRegistry, default server configs
│   ├── config.rs       # LspConfig, LspServerConfig
│   ├── client.rs       # LspClient for server communication
│   ├── types.rs        # Position, Range, TextEdit, WorkspaceEdit
│   ├── rename.rs       # LspRename builder
│   └── installer/      # Auto-download from Mason registry
│       ├── mod.rs      # LspInstaller main logic
│       ├── package.rs  # Package YAML parsing
│       └── platform.rs # OS/arch detection
└── bin/
    └── refactor.rs     # CLI binary
```

## Key Design Patterns

### Builder Pattern (Fluent API)
All main types use builders with method chaining:

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .git(|g| g.branch("main"))
        .files(|f| f.extension("rs")))
    .transform(|t| t.replace_pattern(r"old", "new"))
    .apply()?;
```

### Trait-based Extensibility
- `Language` trait for adding new language support
- `Transform` trait for custom transformations

### Closure-based Configuration
Matchers and transforms accept closures for nested configuration:
```rust
.matching(|m| m.files(|f| f.extension("rs")))
```

## Adding a New Language

1. Create `src/lang/{language}.rs`:
```rust
use super::Language;
use tree_sitter::Language as TsLanguage;

pub struct MyLanguage;

impl Language for MyLanguage {
    fn name(&self) -> &'static str { "mylang" }
    fn extensions(&self) -> &[&'static str] { &["ml", "mli"] }
    fn grammar(&self) -> TsLanguage {
        tree_sitter_mylang::LANGUAGE.into()
    }
}
```

2. Add to `src/lang/mod.rs`:
```rust
mod mylang;
pub use mylang::MyLanguage;
```

3. Register in `LanguageRegistry::new()`:
```rust
registry.register(Box::new(MyLanguage));
```

4. Add dependency to `Cargo.toml`:
```toml
tree-sitter-mylang = "0.x"
```

5. Add tests in `src/lang/mod.rs` tests module.

## Adding a New Transform

1. For text transforms, add variant to `TextTransformKind` in `src/transform/text.rs`
2. For AST transforms, add operation to `AstOperation` in `src/transform/ast.rs`
3. Implement in `Transform::apply()` and `Transform::describe()`
4. Add builder method to `TransformBuilder` in `src/transform/mod.rs`

## Adding a New Matcher Predicate

### Git Predicates (`src/matcher/git.rs`)
1. Add field to `GitMatcher` struct
2. Add builder method
3. Implement check in `matches()` method

### File Predicates (`src/matcher/file.rs`)
1. Add field to `FileMatcher` struct
2. Add builder method
3. Implement filter in `collect()` method

## LSP Refactoring

The LSP module enables semantic refactoring operations via any LSP-compliant language server.

### Using LspRename

```rust
use refactor_dsl::lsp::{LspRename, LspRegistry};

// Rename by finding a symbol
let result = LspRename::find_symbol("src/lib.rs", "old_name", "new_name")?
    .dry_run()  // Preview without applying
    .execute()?;

println!("Would modify {} files", result.file_count());
println!("{}", result.diff()?);

// Rename at specific position (0-indexed line/column)
let result = LspRename::new("src/main.rs", 10, 4, "new_function_name")
    .execute()?;
```

### Supported Language Servers

The `LspRegistry` includes default configurations for:

| Server | Command | Languages |
|--------|---------|-----------|
| rust-analyzer | `rust-analyzer` | Rust (.rs) |
| typescript-language-server | `typescript-language-server --stdio` | TypeScript, JavaScript (.ts, .tsx, .js, .jsx) |
| pyright | `pyright-langserver --stdio` | Python (.py, .pyi) |
| gopls | `gopls serve` | Go (.go) |
| clangd | `clangd` | C/C++ (.c, .cpp, .h, .hpp) |

### Custom LSP Server Configuration

```rust
use refactor_dsl::lsp::{LspServerConfig, LspRename};

let config = LspServerConfig::new("custom-lsp", "my-lsp-server")
    .args(["--stdio"])
    .extensions(["xyz"])
    .root_markers(["config.json"]);

let result = LspRename::new("file.xyz", 0, 0, "new_name")
    .server(config)
    .execute()?;
```

### Adding a New LSP Server

1. Add to `register_defaults()` in `src/lsp/mod.rs`:
```rust
self.register(LspServerConfig {
    name: "new-server".to_string(),
    command: "new-lsp-command".to_string(),
    args: vec!["--stdio".to_string()],
    extensions: vec!["ext".to_string()],
    root_markers: vec!["project.json".to_string()],
});
```

2. Or register at runtime:
```rust
let mut registry = LspRegistry::new();
registry.register(LspServerConfig::new("custom", "custom-lsp")
    .extensions(["custom"]));
```

### LSP Architecture

```
┌──────────────┐       ┌───────────────┐       ┌─────────────────┐
│  LspRename   │──────▶│   LspClient   │──────▶│  Language Server │
│  (builder)   │       │  (JSON-RPC)   │       │  (e.g. rust-analyzer)
└──────────────┘       └───────────────┘       └─────────────────┘
       │                      │
       │                      ▼
       │               ┌───────────────┐
       │               │ WorkspaceEdit │
       │               │   (changes)   │
       │               └───────────────┘
       │                      │
       ▼                      ▼
┌──────────────┐       ┌───────────────┐
│ RenameResult │──────▶│   Apply/Diff  │
└──────────────┘       └───────────────┘
```

The LSP client:
1. Spawns the language server process
2. Sends JSON-RPC `initialize` request
3. Opens the document (`textDocument/didOpen`)
4. Requests rename (`textDocument/rename`)
5. Receives `WorkspaceEdit` with all changes
6. Applies changes or generates diff

### Auto-Installing LSP Servers

The installer module can automatically download LSP servers from the [Mason registry](https://mason-registry.dev):

```rust
use refactor_dsl::lsp::LspInstaller;

// Create installer (uses ~/.local/share/refactor-dsl/lsp-servers by default)
let installer = LspInstaller::new()?;

// Install rust-analyzer if not present
let binary_path = installer.ensure_installed("rust-analyzer")?;
println!("rust-analyzer at: {}", binary_path.display());

// Or use with custom directory
let installer = LspInstaller::new()?
    .install_dir("/opt/lsp-servers");

// List installed servers
for server in installer.list_installed()? {
    println!("{}: {}", server.name, server.binary.display());
}
```

#### Auto-Install with LspRename

```rust
use refactor_dsl::lsp::LspRename;

// Automatically download LSP server if not in PATH
let result = LspRename::find_symbol("src/lib.rs", "old_name", "new_name")?
    .auto_install()  // Downloads server if needed
    .dry_run()
    .execute()?;
```

#### Supported Package Types

The installer supports GitHub release downloads (direct binary):

| Server | Package | Platforms |
|--------|---------|-----------|
| rust-analyzer | `pkg:github/rust-lang/rust-analyzer` | Linux, macOS, Windows (x64, arm64) |
| lua-language-server | `pkg:github/LuaLS/lua-language-server` | Linux, macOS, Windows |
| gopls | `pkg:github/golang/tools/gopls` | Linux, macOS, Windows |

**Note**: npm/pip packages (like `typescript-language-server`, `pyright`) require the respective package manager to be installed.

#### Installer Architecture

```
┌─────────────────┐      ┌──────────────────┐      ┌────────────────────┐
│  LspInstaller   │─────▶│  Mason Registry  │─────▶│  GitHub Releases   │
│                 │      │  (package.yaml)  │      │  (binary download) │
└─────────────────┘      └──────────────────┘      └────────────────────┘
        │                                                    │
        │                                                    ▼
        │                                          ┌────────────────────┐
        │                                          │  Extract (.gz/.zip)│
        │                                          └────────────────────┘
        │                                                    │
        ▼                                                    ▼
┌─────────────────┐                              ┌────────────────────┐
│  ~/.local/share │◀─────────────────────────────│  Install binary    │
│  /refactor-dsl/ │                              │  + info.json       │
│  lsp-servers/   │                              └────────────────────┘
└─────────────────┘
```

## Testing Conventions

- Unit tests go in `#[cfg(test)] mod tests` at bottom of each module
- Integration tests go in `tests/` directory
- Use `tempfile::TempDir` for filesystem tests
- Test both success and error cases
- Test edge cases (empty input, no matches, etc.)

Run tests:
```bash
cargo test                    # All tests
cargo test lang::tests        # Specific module
cargo test --test integration # Integration tests only
```

## Error Handling

- Use `RefactorError` enum for all errors
- Implement `From` for external error types
- Return `Result<T>` (alias for `Result<T, RefactorError>`)
- Prefer `?` operator over `.unwrap()`

## CLI Development

The CLI is in `src/bin/refactor.rs` using clap derive macros:
- Add new subcommands as variants to `Commands` enum
- Implement handler function `cmd_{name}()`
- Keep CLI thin - delegate to library functions

## Common Tasks

### Finding all usages of a function
```bash
cargo test                           # Run tests
rg "function_name" src/              # Search source
```

### Checking tree-sitter queries
Use the tree-sitter CLI or playground to test queries:
```bash
# Parse and show AST
tree-sitter parse src/example.rs

# Test a query
tree-sitter query queries/functions.scm src/example.rs
```

### Tree-sitter Query Syntax
Common patterns:
```scheme
; Match function definitions
(function_item name: (identifier) @fn_name)

; Match struct definitions
(struct_item name: (type_identifier) @struct_name)

; Match function calls
(call_expression function: (identifier) @fn_call)

; Match with predicates
((identifier) @id (#eq? @id "specific_name"))
```

## Dependencies

| Crate | Purpose |
|-------|---------|
| `tree-sitter` | AST parsing |
| `tree-sitter-{lang}` | Language grammars |
| `git2` | Git operations |
| `walkdir` | Directory traversal |
| `globset` | Glob pattern matching |
| `regex` | Pattern matching |
| `similar` | Diff generation |
| `clap` | CLI argument parsing |
| `thiserror` | Error derive macros |
| `anyhow` | Error handling in CLI |
| `serde` | Serialization |
| `lsp-types` | LSP protocol types |
| `url` | URL handling for LSP |
| `tokio` | Async runtime (LSP) |
| `tower-lsp` | LSP server framework |
| `reqwest` | HTTP client (installer) |
| `serde_yaml` | YAML parsing (installer) |
| `flate2` | Gzip decompression |
| `zip` | ZIP extraction |
| `tar` | Tar archive handling |
| `dirs` | Platform directories |

## Performance Considerations

- File content matching (`contains_pattern`) reads entire files - use last in filter chain
- AST parsing is expensive - cache trees when processing same file multiple times
- Use `dry_run()` for previewing changes without disk I/O
- `FileMatcher::collect()` walks entire directory tree - use specific paths when possible

## Future Enhancements

Potential areas for extension:
- [ ] Parallel file processing
- [ ] Incremental/cached parsing
- [ ] YAML/TOML configuration file support
- [ ] More tree-sitter languages (Java, Ruby, etc.)
- [x] Semantic rename (LSP-based cross-file symbol tracking)
- [x] Auto-download LSP servers from Mason registry
- [ ] npm/pip package installation for LSP servers
- [ ] LSP find-references integration
- [ ] LSP go-to-definition integration
- [ ] Interactive mode with preview/confirm
- [ ] VSCode extension integration
- [ ] LSP code actions support
