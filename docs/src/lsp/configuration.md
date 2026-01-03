# LSP Configuration

Configure LSP servers for different languages using `LspRegistry` and `LspServerConfig`.

## LspRegistry

The registry manages server configurations for different languages:

```rust
use refactor::lsp::{LspRegistry, LspServerConfig};

// Create with defaults (rust-analyzer, tsserver, pyright, etc.)
let registry = LspRegistry::new();

// Create empty registry
let registry = LspRegistry::empty();
```

## Finding Servers

```rust
let registry = LspRegistry::new();

// By file extension
let config = registry.find_by_extension("rs");

// By file path
let config = registry.find_for_file(Path::new("src/main.rs"));

// List all registered servers
for server in registry.all() {
    println!("{}: {}", server.name, server.command);
}
```

## Default Servers

`LspRegistry::new()` includes these defaults:

| Name | Command | Extensions | Root Markers |
|------|---------|------------|--------------|
| rust-analyzer | `rust-analyzer` | `rs` | `Cargo.toml`, `rust-project.json` |
| typescript-language-server | `typescript-language-server --stdio` | `ts`, `tsx`, `js`, `jsx` | `tsconfig.json`, `jsconfig.json`, `package.json` |
| pyright | `pyright-langserver --stdio` | `py`, `pyi` | `pyproject.toml`, `setup.py`, `pyrightconfig.json` |
| gopls | `gopls serve` | `go` | `go.mod`, `go.work` |
| clangd | `clangd` | `c`, `h`, `cpp`, `hpp`, `cc`, `cxx` | `compile_commands.json`, `CMakeLists.txt`, `.clangd` |

## Custom Server Configuration

### LspServerConfig Builder

```rust
use refactor::lsp::LspServerConfig;

let config = LspServerConfig::new("my-lsp", "/path/to/my-lsp")
    .arg("--stdio")
    .arg("--verbose")
    .extensions(["myext", "myx"])
    .root_markers(["myproject.json", ".myconfig"]);
```

### Register Custom Servers

```rust
let mut registry = LspRegistry::new();

registry.register(
    LspServerConfig::new("custom-lsp", "custom-language-server")
        .arg("--stdio")
        .extensions(["custom"])
        .root_markers(["custom.config"])
);

// Now works with .custom files
let config = registry.find_by_extension("custom");
```

## Root Detection

LSP servers need to know the project root. Root markers help find it:

```rust
let config = LspServerConfig::new("rust-analyzer", "rust-analyzer")
    .root_markers(["Cargo.toml", "rust-project.json"]);

// Searches upward from file path to find root
let root = config.find_root(Path::new("src/main.rs"));
// Returns Some("/path/to/project") if Cargo.toml found
```

## Using with LspClient

```rust
use refactor::lsp::{LspClient, LspRegistry};

let registry = LspRegistry::new();
let config = registry.find_for_file(Path::new("src/main.rs"))
    .expect("No LSP for Rust files");

let root = config.find_root(Path::new("src/main.rs"))
    .unwrap_or_else(|| PathBuf::from("."));

let mut client = LspClient::start(&config, &root)?;
client.initialize()?;
```

## Using with LspRename

```rust
use refactor::lsp::{LspRename, LspServerConfig};

// Use default server (auto-detected)
let result = LspRename::new("src/main.rs", 10, 4, "new_name")
    .execute()?;

// Use custom server
let custom = LspServerConfig::new("my-analyzer", "/opt/my-analyzer")
    .arg("--stdio");

let result = LspRename::new("src/main.rs", 10, 4, "new_name")
    .server(custom)
    .execute()?;
```

## Environment Requirements

LSP servers must be:

1. **Installed and in PATH** - Or provide absolute path
2. **Executable** - Proper permissions set
3. **Compatible** - Support the LSP protocol

Check server availability:

```bash
# Rust
which rust-analyzer

# TypeScript
which typescript-language-server

# Python
which pyright-langserver
```

If not installed, use [auto-installation](./auto-install.md).

## Troubleshooting

### Server Not Found

```rust
// Check if server command exists
use std::process::Command;

fn server_exists(command: &str) -> bool {
    Command::new(command)
        .arg("--version")
        .output()
        .is_ok()
}
```

### Wrong Root Detection

```rust
// Explicitly set the root
LspRename::new("src/main.rs", 10, 4, "new_name")
    .root("/path/to/project")
    .execute()?;
```

### Server Crashes

Enable debugging by checking server stderr (currently discarded):

```rust
// Modify LspClient::start to capture stderr for debugging
// (Feature enhancement planned)
```
