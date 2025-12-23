# LSP Auto-Installation

Refactor DSL can automatically download and install LSP servers from the [Mason registry](https://mason-registry.dev/), the same registry used by Neovim's mason.nvim.

## Quick Start

```rust
use refactor_dsl::lsp::LspRename;

// Automatically install rust-analyzer if not in PATH
LspRename::find_symbol("src/main.rs", "old_fn", "new_fn")?
    .auto_install()
    .execute()?;
```

## LspInstaller

For direct control over installations:

```rust
use refactor_dsl::lsp::LspInstaller;

let installer = LspInstaller::new()?;

// Install a server
let binary_path = installer.install("rust-analyzer")?;
println!("Installed to: {}", binary_path.display());

// Check if already installed
if installer.is_installed("rust-analyzer") {
    println!("rust-analyzer is ready");
}

// Get path to installed binary
let path = installer.get_binary_path("rust-analyzer");
```

## Available Servers

The installer supports any server in the Mason registry. Common ones:

| Package Name | Language |
|--------------|----------|
| `rust-analyzer` | Rust |
| `typescript-language-server` | TypeScript/JavaScript |
| `pyright` | Python |
| `gopls` | Go |
| `clangd` | C/C++ |
| `lua-language-server` | Lua |
| `yaml-language-server` | YAML |
| `json-lsp` | JSON |

Browse all packages at [mason-registry.dev](https://mason-registry.dev/).

## Installation Directory

By default, servers are installed to:

- **Linux/macOS:** `~/.local/share/refactor-dsl/lsp-servers/`
- **Windows:** `%LOCALAPPDATA%/refactor-dsl/lsp-servers/`

Customize the location:

```rust
let installer = LspInstaller::new()?
    .install_dir("/opt/lsp-servers")
    .cache_dir("/tmp/lsp-cache");
```

## Platform Detection

The installer automatically detects your platform:

```rust
use refactor_dsl::lsp::installer::{Platform, Os, Arch};

let platform = Platform::detect();
println!("OS: {:?}, Arch: {:?}", platform.os, platform.arch);
```

Supported platforms:
- **OS:** Linux, macOS, Windows
- **Arch:** x64, arm64, x86

## Listing Installed Servers

```rust
let installer = LspInstaller::new()?;

for server in installer.list_installed()? {
    println!("{} v{} at {}",
        server.name,
        server.version,
        server.binary.display());
}
```

## Uninstalling

```rust
installer.uninstall("rust-analyzer")?;
```

## Ensure Installed

Install only if not already present:

```rust
// Returns path to binary, installing if needed
let binary = installer.ensure_installed("rust-analyzer")?;
```

## How It Works

1. **Fetch metadata** from Mason registry (`package.yaml`)
2. **Select asset** for current platform
3. **Download** binary/archive to cache
4. **Extract** (supports `.gz`, `.tar.gz`, `.zip`)
5. **Install** to installation directory
6. **Set permissions** (executable on Unix)

```
Mason Registry (GitHub)
        │
        ▼
┌───────────────┐
│ package.yaml  │ ← Package metadata
└───────────────┘
        │
        ▼
┌───────────────┐
│ GitHub Release│ ← Binary download
└───────────────┘
        │
        ▼
┌───────────────┐
│ Local Cache   │ ← Downloaded archive
└───────────────┘
        │
        ▼
┌───────────────┐
│ Install Dir   │ ← Extracted binary
└───────────────┘
```

## Integration with LspRename

The `auto_install()` method on `LspRename` uses the installer:

```rust
LspRename::new("src/main.rs", 10, 4, "new_name")
    .auto_install()  // Enables auto-installation
    .execute()?;
```

This will:
1. Detect language from file extension
2. Find appropriate LSP server
3. Check if server exists in PATH
4. If not, install from Mason registry
5. Update config to use installed binary
6. Proceed with rename operation

## Error Handling

```rust
use refactor_dsl::error::RefactorError;

match installer.install("unknown-server") {
    Ok(path) => println!("Installed to {}", path.display()),
    Err(RefactorError::TransformFailed { message }) => {
        if message.contains("not found in registry") {
            println!("Package doesn't exist in Mason registry");
        } else if message.contains("No binary available") {
            println!("No binary for this platform");
        } else {
            println!("Installation failed: {}", message);
        }
    }
    Err(e) => return Err(e.into()),
}
```

## Offline Usage

If you need to work offline:

1. Pre-install servers while online
2. Use explicit binary paths in `LspServerConfig`
3. Disable `auto_install()`

```rust
// Use pre-installed server
let config = LspServerConfig::new(
    "rust-analyzer",
    "/path/to/rust-analyzer"
);

LspRename::new("src/main.rs", 10, 4, "new_name")
    .server(config)  // Explicit server, no auto-install
    .execute()?;
```
