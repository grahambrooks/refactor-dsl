# Move Operations

Move operations relocate code between files and modules while maintaining correctness by updating imports and references.

## MoveToFile

Move a symbol (function, struct, class, etc.) to a different file.

### Basic Usage

```rust
use refactor::prelude::*;

// Move a function to another file
let result = MoveToFile::new("process_data")
    .from_file("src/utils.rs")
    .to_file("src/processors.rs")
    .execute()?;

println!("Moved to: {}", result.target_file.display());
```

### Configuration

```rust
MoveToFile::new("DataProcessor")
    .from_file("src/lib.rs")
    .to_file("src/processing/mod.rs")
    // Update all imports across the project
    .update_imports(true)
    // Add re-export from original location for backwards compatibility
    .add_reexport(true)
    // Move related items (e.g., impl blocks for a struct)
    .include_related(true)
    // Preview first
    .dry_run()
    .execute()?;
```

### Example

```rust
// Before (src/utils.rs):
pub fn process_data(input: &str) -> Result<Data> {
    // ...
}

pub fn other_util() { /* ... */ }

// src/main.rs:
use crate::utils::process_data;

fn main() {
    let result = process_data("input");
}

// After moving `process_data` to src/processors.rs:

// src/processors.rs (new or updated):
pub fn process_data(input: &str) -> Result<Data> {
    // ...
}

// src/utils.rs:
pub fn other_util() { /* ... */ }

// src/main.rs (imports updated):
use crate::processors::process_data;

fn main() {
    let result = process_data("input");
}
```

### Moving with Related Items

When moving a struct, you often want to move its impl blocks too:

```rust
MoveToFile::new("User")
    .from_file("src/models.rs")
    .to_file("src/user/mod.rs")
    .include_related(true)  // Moves struct + all impl blocks
    .execute()?;
```

## MoveBetweenModules

Move code between modules with proper path updates.

### Basic Usage

```rust
use refactor::prelude::*;

// Move to a different module path
let result = MoveBetweenModules::new("DataProcessor")
    .from_module("crate::utils")
    .to_module("crate::processors")
    .execute()?;
```

### Configuration

```rust
MoveBetweenModules::new("Config")
    .from_module("crate::settings")
    .to_module("crate::config::types")
    // Update all references in the codebase
    .update_references(true)
    // Add a re-export from the old path
    .add_reexport(true)
    // Create the target module if it doesn't exist
    .create_target_module(true)
    .execute()?;
```

### Example

```rust
// Before:
// crate::utils::helpers

pub struct DataProcessor { /* ... */ }

// Usage in another file:
use crate::utils::helpers::DataProcessor;

// After moving to crate::processing::core:

// crate::processing::core
pub struct DataProcessor { /* ... */ }

// Usage updated:
use crate::processing::core::DataProcessor;
```

## Import Updates

Move operations automatically update imports:

### Rust

```rust
// Before:
use crate::utils::process_data;
use crate::utils::{format, validate};

// After moving process_data to crate::processors:
use crate::processors::process_data;
use crate::utils::{format, validate};
```

### TypeScript

```typescript
// Before:
import { processData, format } from './utils';

// After moving processData to ./processors:
import { processData } from './processors';
import { format } from './utils';
```

### Python

```python
# Before:
from utils import process_data, format_data

# After moving process_data to processors:
from processors import process_data
from utils import format_data
```

## Re-exports for Compatibility

To maintain backwards compatibility, add re-exports:

```rust
MoveToFile::new("legacy_api")
    .from_file("src/lib.rs")
    .to_file("src/legacy/mod.rs")
    .add_reexport(true)
    .execute()?;
```

This creates:

```rust
// src/lib.rs
pub use crate::legacy::legacy_api;  // Re-export for compatibility

// src/legacy/mod.rs
pub fn legacy_api() { /* ... */ }
```

## Validation

Move operations validate:

- Target file/module exists or can be created
- No name conflicts at destination
- All references can be updated
- Circular dependencies won't be introduced

```rust
let validation = MoveToFile::new("helper")
    .from_file("src/utils.rs")
    .to_file("src/helpers.rs")
    .validate()?;

match validation {
    ValidationResult::Valid => println!("Ready to move"),
    ValidationResult::Warning(msg) => {
        println!("Warning: {}", msg);
        // e.g., "Target file will be created"
    }
    ValidationResult::Invalid(msg) => {
        println!("Cannot move: {}", msg);
        // e.g., "Would create circular dependency"
    }
}
```

## Error Handling

```rust
use refactor::error::RefactorError;

match MoveToFile::new("function").from_file("src/a.rs").to_file("src/b.rs").execute() {
    Ok(result) => {
        println!("Moved successfully");
        println!("Updated {} imports", result.imports_updated);
    }
    Err(RefactorError::SymbolNotFound(name)) => {
        println!("Symbol '{}' not found in source file", name);
    }
    Err(RefactorError::NameConflict(name)) => {
        println!("'{}' already exists in target file", name);
    }
    Err(RefactorError::CircularDependency(msg)) => {
        println!("Would create circular dependency: {}", msg);
    }
    Err(e) => return Err(e.into()),
}
```

## Language Support

| Language   | MoveToFile | MoveBetweenModules |
|------------|------------|---------------------|
| Rust       | Yes        | Yes                 |
| TypeScript | Yes        | Yes                 |
| Python     | Yes        | Yes                 |
| Go         | Yes        | Yes                 |
| Java       | Yes        | Yes                 |
| C#         | Yes        | Yes                 |
| Ruby       | Yes        | Limited             |

## Best Practices

1. **Preview changes** - Always use `.dry_run()` first
2. **Add re-exports initially** - Remove them after migration period
3. **Move related items together** - Use `.include_related(true)`
4. **Update tests** - Move operations update source files, check test imports
5. **Commit before moving** - Have a clean git state to easily revert

## See Also

- [Change Signature](./signature.md) - Modify function signatures
- [Safe Delete](./safe-delete.md) - Remove unused code after moving
