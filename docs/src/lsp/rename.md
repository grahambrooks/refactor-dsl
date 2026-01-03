# Semantic Rename

`LspRename` provides semantic symbol renaming that understands code structure, updates all references, and modifies imports across files.

## Basic Usage

### By Position

Rename the symbol at a specific location:

```rust
use refactor::lsp::LspRename;

// Rename symbol at line 10, column 4 (0-indexed)
let result = LspRename::new("src/main.rs", 10, 4, "new_name")
    .execute()?;

println!("Modified {} files", result.file_count());
```

### By Symbol Name

Find and rename a symbol by name:

```rust
// Find first occurrence of "old_function" and rename it
let result = LspRename::find_symbol("src/lib.rs", "old_function", "new_function")?
    .execute()?;
```

## Builder Methods

### Project Root

Set the project root explicitly:

```rust
LspRename::new("src/main.rs", 10, 4, "new_name")
    .root("/path/to/project")
    .execute()?;
```

### Custom LSP Server

Use a specific server configuration:

```rust
use refactor::lsp::LspServerConfig;

let config = LspServerConfig::new("my-analyzer", "/opt/my-analyzer")
    .arg("--stdio");

LspRename::new("src/main.rs", 10, 4, "new_name")
    .server(config)
    .execute()?;
```

### Auto-Installation

Automatically download the LSP server if needed:

```rust
LspRename::new("src/main.rs", 10, 4, "new_name")
    .auto_install()
    .execute()?;
```

### Dry Run

Preview changes without applying:

```rust
let result = LspRename::new("src/main.rs", 10, 4, "new_name")
    .dry_run()
    .execute()?;

// Show what would change
println!("{}", result.diff()?);
```

## RenameResult

The result provides information about the changes:

```rust
let result = LspRename::new("src/main.rs", 10, 4, "new_name")
    .execute()?;

// Number of files affected
println!("Files: {}", result.file_count());

// Total number of edits
println!("Edits: {}", result.edit_count());

// Check if empty (symbol not found or not renameable)
if result.is_empty() {
    println!("No changes made");
}

// Generate unified diff
println!("{}", result.diff()?);

// Was this a dry run?
if result.dry_run {
    println!("Changes not applied (dry run)");
}
```

## WorkspaceEdit

The underlying `WorkspaceEdit` contains all changes:

```rust
let edit = &result.workspace_edit;

// Iterate over file changes
for (path, edits) in edit.changes() {
    println!("{}:", path.display());
    for e in edits {
        println!("  Line {}: {} -> {}",
            e.range.start.line,
            e.range,
            e.new_text);
    }
}

// Preview new content without applying
let previews = edit.preview()?;
for (path, new_content) in &previews {
    println!("=== {} ===", path.display());
    println!("{}", new_content);
}

// Apply changes (already done if not dry_run)
edit.apply()?;
```

## Complete Example

```rust
use refactor::lsp::LspRename;

fn rename_api_function() -> Result<()> {
    // Find the function to rename
    let result = LspRename::find_symbol(
        "src/api/handlers.rs",
        "handle_request",
        "process_request"
    )?
    .root("./my-project")
    .auto_install()
    .dry_run()
    .execute()?;

    if result.is_empty() {
        println!("Symbol not found or not renameable");
        return Ok(());
    }

    println!("Preview of changes:");
    println!("{}", result.diff()?);
    println!("\nWould modify {} files with {} edits",
        result.file_count(),
        result.edit_count());

    // Ask for confirmation, then apply
    println!("\nApply changes? [y/N]");
    // ... get user input ...

    // Apply without dry_run
    LspRename::find_symbol(
        "src/api/handlers.rs",
        "handle_request",
        "process_request"
    )?
    .root("./my-project")
    .execute()?;

    println!("Done!");
    Ok(())
}
```

## What Gets Renamed

A semantic rename updates:

- **Function/method definitions**
- **Function/method calls**
- **Variable declarations and usages**
- **Type definitions and references**
- **Import/export statements**
- **Documentation references** (server-dependent)

Example with Rust:

```rust
// Before: rename `process` to `handle`

// src/lib.rs
pub fn process(data: &str) -> String { ... }

// src/main.rs
use mylib::process;

fn main() {
    let result = process("input");
}
```

```rust
// After

// src/lib.rs
pub fn handle(data: &str) -> String { ... }

// src/main.rs
use mylib::handle;

fn main() {
    let result = handle("input");
}
```

## Error Handling

```rust
use refactor::error::RefactorError;

match LspRename::find_symbol("src/main.rs", "not_found", "new_name") {
    Ok(rename) => {
        match rename.execute() {
            Ok(result) if result.is_empty() => {
                println!("Symbol found but not renameable");
            }
            Ok(result) => {
                println!("Renamed in {} files", result.file_count());
            }
            Err(RefactorError::TransformFailed { message }) => {
                println!("LSP error: {}", message);
            }
            Err(e) => return Err(e.into()),
        }
    }
    Err(RefactorError::TransformFailed { message }) => {
        println!("Symbol not found: {}", message);
    }
    Err(e) => return Err(e.into()),
}
```

## Limitations

- **Requires LSP server** for the language
- **Single symbol at a time** (no batch rename)
- **Server must support rename** (most do)
- **May miss dynamic references** (reflection, eval, etc.)

## See Also

- [LSP Configuration](./configuration.md)
- [Auto-Installation](./auto-install.md)
