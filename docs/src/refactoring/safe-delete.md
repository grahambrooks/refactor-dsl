# Safe Delete

Safe delete operations remove code while checking for usages, preventing accidental breakage.

## Overview

Unlike regular deletion, `SafeDelete`:

- **Checks for usages** - Warns if the symbol is still referenced
- **Shows affected locations** - Lists all files that would break
- **Supports cascade** - Optionally delete dependent code
- **Provides alternatives** - Suggests refactoring options

## Basic Usage

```rust
use refactor::prelude::*;

// Delete a function, checking for usages first
let result = SafeDelete::symbol("unused_helper")
    .in_file("src/utils.rs")
    .check_usages(true)
    .execute()?;

match result {
    DeleteResult::Deleted => println!("Successfully deleted"),
    DeleteResult::HasUsages(usages) => {
        println!("Cannot delete: {} usages found", usages.len());
        for usage in usages {
            println!("  - {}:{}", usage.file.display(), usage.line);
        }
    }
}
```

## Delete Types

### Functions

```rust
SafeDelete::function("helper_function")
    .in_file("src/utils.rs")
    .execute()?;
```

### Types (Struct, Class, Enum)

```rust
SafeDelete::type_def("OldConfig")
    .in_file("src/config.rs")
    .execute()?;
```

### Methods

```rust
SafeDelete::method("deprecated_method")
    .on_type("MyStruct")
    .in_file("src/lib.rs")
    .execute()?;
```

### Variables/Constants

```rust
SafeDelete::constant("UNUSED_CONST")
    .in_file("src/constants.rs")
    .execute()?;
```

### Imports

```rust
SafeDelete::import("unused_module")
    .in_file("src/main.rs")
    .execute()?;
```

## Cascade Delete

Delete a symbol and all code that depends on it:

```rust
SafeDelete::symbol("deprecated_module")
    .in_file("src/lib.rs")
    .cascade(true)
    .execute()?;
```

**Warning:** Cascade delete can remove significant amounts of code. Always preview first!

### Cascade Example

```rust
// If we delete `HelperTrait`:

pub trait HelperTrait {
    fn help(&self);
}

impl HelperTrait for MyStruct {  // Would also be deleted
    fn help(&self) { /* ... */ }
}

fn use_helper<T: HelperTrait>(t: T) {  // Would also be deleted
    t.help();
}
```

## Preview Cascade

See what would be deleted without doing it:

```rust
let preview = SafeDelete::symbol("deprecated_api")
    .in_file("src/lib.rs")
    .cascade(true)
    .preview()?;

println!("Would delete {} items:", preview.items.len());
for item in &preview.items {
    println!("  - {} ({}) at {}:{}",
        item.name,
        item.kind,
        item.file.display(),
        item.line);
}

// Optionally proceed
if user_confirmed() {
    SafeDelete::symbol("deprecated_api")
        .in_file("src/lib.rs")
        .cascade(true)
        .execute()?;
}
```

## Force Delete

Delete even if there are usages (use with caution):

```rust
SafeDelete::symbol("must_remove")
    .in_file("src/lib.rs")
    .force(true)  // Delete despite usages
    .execute()?;
```

This will delete the symbol and leave broken references. Use this only when you plan to fix references manually.

## Search Scope

Control where to search for usages:

```rust
SafeDelete::symbol("internal_helper")
    .in_file("src/utils.rs")
    // Only search in specific directory
    .search_scope("src/internal")
    .execute()?;

// Or exclude certain paths
SafeDelete::symbol("helper")
    .in_file("src/lib.rs")
    .exclude_from_search("**/tests/**")
    .exclude_from_search("**/examples/**")
    .execute()?;
```

## Handling Results

```rust
let result = SafeDelete::symbol("maybe_used")
    .in_file("src/lib.rs")
    .check_usages(true)
    .execute()?;

match result {
    DeleteResult::Deleted => {
        println!("Symbol deleted successfully");
    }
    DeleteResult::HasUsages(usages) => {
        println!("Found {} usages:", usages.len());
        for usage in usages {
            println!("  {}:{}:{} - {}",
                usage.file.display(),
                usage.line,
                usage.column,
                usage.context);  // Surrounding code
        }

        // Options:
        // 1. Fix usages manually, then delete
        // 2. Use cascade to delete dependent code
        // 3. Force delete and fix later
    }
    DeleteResult::NotFound => {
        println!("Symbol not found");
    }
}
```

## Error Handling

```rust
use refactor::error::RefactorError;

match SafeDelete::symbol("item").in_file("src/lib.rs").execute() {
    Ok(DeleteResult::Deleted) => println!("Success"),
    Ok(DeleteResult::HasUsages(usages)) => {
        println!("Has {} usages", usages.len());
    }
    Ok(DeleteResult::NotFound) => {
        println!("Symbol not found");
    }
    Err(RefactorError::SymbolNotFound(name)) => {
        println!("File doesn't contain '{}'", name);
    }
    Err(RefactorError::AmbiguousReference(msg)) => {
        println!("Multiple symbols match: {}", msg);
    }
    Err(e) => return Err(e.into()),
}
```

## Language Support

| Language   | Function | Type | Method | Variable | Import |
|------------|----------|------|--------|----------|--------|
| Rust       | Yes      | Yes  | Yes    | Yes      | Yes    |
| TypeScript | Yes      | Yes  | Yes    | Yes      | Yes    |
| Python     | Yes      | Yes  | Yes    | Yes      | Yes    |
| Go         | Yes      | Yes  | Yes    | Yes      | Yes    |
| Java       | Yes      | Yes  | Yes    | Yes      | Yes    |
| C#         | Yes      | Yes  | Yes    | Yes      | Yes    |
| Ruby       | Yes      | Yes  | Yes    | Yes      | Yes    |

## Integration with Other Operations

### After Inline

```rust
// Inline a function, then safely delete it
InlineFunction::new("helper")
    .in_file("src/utils.rs")
    .all_call_sites(true)
    .execute()?;

SafeDelete::function("helper")
    .in_file("src/utils.rs")
    .execute()?;  // Should have no usages now
```

### After Move

```rust
// Move a function, then delete the old location's re-export
MoveToFile::new("process")
    .from_file("src/old.rs")
    .to_file("src/new.rs")
    .add_reexport(false)  // Don't add re-export
    .execute()?;

// Or if re-export was added initially, remove it later
SafeDelete::import("process")
    .in_file("src/old.rs")
    .execute()?;
```

## Best Practices

1. **Always preview first** - Use `.check_usages(true)` before deletion
2. **Prefer cascade sparingly** - Review what will be deleted
3. **Check tests** - Tests may have usages not in main source
4. **Consider deprecation first** - Mark as deprecated before deleting
5. **Keep git clean** - Easy to revert if something breaks

## See Also

- [Find Dead Code](./dead-code.md) - Identify unused code to delete
- [Inline](./inline.md) - Inline before deleting
