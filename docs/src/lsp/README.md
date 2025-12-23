# LSP Integration

Refactor DSL integrates with Language Server Protocol (LSP) servers to provide semantic refactoring capabilities that understand your code's types, scopes, and references.

## Why LSP?

While text-based and AST transforms are powerful, they lack semantic understanding:

| Feature | Text/AST | LSP |
|---------|----------|-----|
| Find text patterns | Yes | Yes |
| Understand syntax | AST only | Yes |
| Understand types | No | Yes |
| Cross-file references | No | Yes |
| Rename with imports | No | Yes |
| Find all usages | Limited | Yes |

## Quick Start

```rust
use refactor_dsl::lsp::LspRename;

// Rename a symbol semantically
let result = LspRename::new("src/main.rs", 10, 4, "new_function_name")
    .auto_install()  // Download LSP server if needed
    .dry_run()       // Preview changes
    .execute()?;

println!("Would modify {} files:", result.file_count());
println!("{}", result.diff()?);
```

## Supported Languages

Out of the box, Refactor DSL supports:

| Language | LSP Server | Extensions |
|----------|------------|------------|
| Rust | rust-analyzer | `.rs` |
| TypeScript/JavaScript | typescript-language-server | `.ts`, `.tsx`, `.js`, `.jsx` |
| Python | pyright | `.py`, `.pyi` |
| Go | gopls | `.go` |
| C/C++ | clangd | `.c`, `.h`, `.cpp`, `.hpp`, `.cc`, `.cxx` |

## Components

### [LspRegistry](./configuration.md)

Manages LSP server configurations for different languages:

```rust
let registry = LspRegistry::new();  // Includes defaults
let config = registry.find_for_file(Path::new("src/main.rs"));
```

### [LspInstaller](./auto-install.md)

Downloads and installs LSP servers from the Mason registry:

```rust
let installer = LspInstaller::new()?;
let binary = installer.install("rust-analyzer")?;
```

### [LspRename](./rename.md)

Performs semantic rename operations:

```rust
LspRename::find_symbol("src/lib.rs", "old_name", "new_name")?
    .execute()?;
```

### LspClient

Low-level client for LSP communication:

```rust
let mut client = LspClient::start(&config, &root_path)?;
client.initialize()?;
client.open_document(path)?;
let edit = client.rename(path, position, "new_name")?;
```

## Architecture

```
┌─────────────────────────────────────────────────┐
│                  LspRename                       │
│  (High-level semantic rename API)               │
├─────────────────────────────────────────────────┤
│                                                  │
│  ┌─────────────┐  ┌─────────────┐               │
│  │ LspRegistry │  │ LspInstaller│               │
│  │             │  │             │               │
│  │ Find server │  │ Download    │               │
│  │ for file    │  │ from Mason  │               │
│  └─────────────┘  └─────────────┘               │
│                                                  │
├─────────────────────────────────────────────────┤
│                  LspClient                       │
│  (JSON-RPC communication with LSP server)       │
├─────────────────────────────────────────────────┤
│                                                  │
│  rust-analyzer │ tsserver │ pyright │ ...       │
│                                                  │
└─────────────────────────────────────────────────┘
```

## Capabilities

Currently supported LSP operations:

- **Rename** - Rename symbols across files with import updates
- **Find References** - Locate all usages of a symbol
- **Go to Definition** - Find symbol definitions

Future planned operations:

- Extract function/method
- Inline variable
- Move to file
- Organize imports

## Error Handling

```rust
use refactor_dsl::error::RefactorError;

match LspRename::new("file.rs", 10, 4, "new_name").execute() {
    Ok(result) => println!("Renamed in {} files", result.file_count()),
    Err(RefactorError::UnsupportedLanguage(ext)) => {
        println!("No LSP server for .{} files", ext);
    }
    Err(RefactorError::TransformFailed { message }) => {
        println!("LSP error: {}", message);
    }
    Err(e) => return Err(e.into()),
}
```

## See Also

- [Configuration](./configuration.md) - Configure LSP servers
- [Auto-Installation](./auto-install.md) - Automatic server downloads
- [Semantic Rename](./rename.md) - Rename symbols safely
