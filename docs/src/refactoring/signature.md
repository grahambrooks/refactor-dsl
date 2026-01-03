# Change Signature

Change signature operations modify function signatures while automatically updating all call sites to match.

## Overview

`ChangeSignature` allows you to:

- Add, remove, or rename parameters
- Reorder parameters
- Change return types
- Add default values for new parameters

All call sites are updated automatically to maintain correctness.

## Basic Usage

```rust
use refactor::prelude::*;

// Add a new parameter with a default value
let result = ChangeSignature::for_function("process")
    .in_file("src/lib.rs")
    .add_parameter("timeout", "Duration", "Duration::from_secs(30)")
    .execute()?;

println!("Updated {} call sites", result.call_sites_updated);
```

## Adding Parameters

### With Default Value

```rust
ChangeSignature::for_function("connect")
    .in_file("src/client.rs")
    .add_parameter("retry_count", "u32", "3")
    .execute()?;
```

Before:

```rust
fn connect(host: &str) -> Result<Connection> { /* ... */ }

// Call sites:
let conn = connect("localhost")?;
```

After:

```rust
fn connect(host: &str, retry_count: u32) -> Result<Connection> { /* ... */ }

// Call sites updated:
let conn = connect("localhost", 3)?;
```

### At Specific Position

```rust
ChangeSignature::for_function("create_user")
    .in_file("src/users.rs")
    .add_parameter_at("email", "String", 1)  // After first param
    .default_value("String::new()")
    .execute()?;
```

### Multiple Parameters

```rust
ChangeSignature::for_function("configure")
    .in_file("src/config.rs")
    .add_parameter("debug", "bool", "false")
    .add_parameter("verbose", "bool", "false")
    .add_parameter("timeout", "Duration", "Duration::from_secs(60)")
    .execute()?;
```

## Removing Parameters

```rust
ChangeSignature::for_function("old_api")
    .in_file("src/legacy.rs")
    .remove_parameter("unused_flag")
    .execute()?;
```

Before:

```rust
fn old_api(data: &str, unused_flag: bool) -> String { /* ... */ }

// Call sites:
let result = old_api("input", true);
let result = old_api("other", false);
```

After:

```rust
fn old_api(data: &str) -> String { /* ... */ }

// Call sites updated:
let result = old_api("input");
let result = old_api("other");
```

## Renaming Parameters

```rust
ChangeSignature::for_function("process")
    .in_file("src/processor.rs")
    .rename_parameter("input", "source")
    .execute()?;
```

This updates:

- The parameter name in the function definition
- All uses of the parameter within the function body
- Named arguments at call sites (for languages that support them)

## Reordering Parameters

```rust
ChangeSignature::for_function("create_user")
    .in_file("src/users.rs")
    .reorder_parameters(&["name", "email", "role"])
    .execute()?;
```

Before:

```rust
fn create_user(role: Role, name: String, email: String) -> User { /* ... */ }

// Call sites:
create_user(Role::Admin, "Alice".into(), "alice@example.com".into())
```

After:

```rust
fn create_user(name: String, email: String, role: Role) -> User { /* ... */ }

// Call sites reordered:
create_user("Alice".into(), "alice@example.com".into(), Role::Admin)
```

## Changing Types

```rust
ChangeSignature::for_function("process")
    .in_file("src/lib.rs")
    .change_parameter_type("count", "u32", "usize")
    .execute()?;
```

**Note:** Type changes may require manual updates if the new type is incompatible with existing usage.

## Complex Changes

Combine multiple modifications:

```rust
ChangeSignature::for_function("legacy_handler")
    .in_file("src/handlers.rs")
    // Remove deprecated parameters
    .remove_parameter("deprecated_flag")
    .remove_parameter("old_config")
    // Add new parameters
    .add_parameter("options", "Options", "Options::default()")
    // Rename for clarity
    .rename_parameter("cb", "callback")
    // Reorder for consistency
    .reorder_parameters(&["request", "options", "callback"])
    .execute()?;
```

## Method Signatures

For methods on structs/classes:

```rust
// Rust
ChangeSignature::for_method("process")
    .on_type("DataProcessor")
    .in_file("src/processor.rs")
    .add_parameter("config", "&Config", "&Config::default()")
    .execute()?;

// Java
ChangeSignature::for_method("process")
    .on_type("DataProcessor")
    .in_file("src/DataProcessor.java")
    .add_parameter("config", "Config", "new Config()")
    .execute()?;
```

## Validation

```rust
let validation = ChangeSignature::for_function("api")
    .in_file("src/lib.rs")
    .add_parameter("new_param", "String", "String::new()")
    .validate()?;

match validation {
    ValidationResult::Valid => println!("Ready to change signature"),
    ValidationResult::Warning(msg) => {
        println!("Warning: {}", msg);
        // e.g., "Found 50 call sites to update"
    }
    ValidationResult::Invalid(msg) => {
        println!("Cannot change signature: {}", msg);
        // e.g., "Function is part of a trait implementation"
    }
}
```

## Error Handling

```rust
use refactor::error::RefactorError;

match ChangeSignature::for_function("api").in_file("src/lib.rs")
    .add_parameter("param", "Type", "default")
    .execute()
{
    Ok(result) => {
        println!("Updated {} call sites", result.call_sites_updated);
    }
    Err(RefactorError::SymbolNotFound(name)) => {
        println!("Function '{}' not found", name);
    }
    Err(RefactorError::TraitConstraint(msg)) => {
        println!("Cannot change: {}", msg);
        // Signature must match trait definition
    }
    Err(RefactorError::AmbiguousReference(msg)) => {
        println!("Ambiguous function: {}", msg);
        // Multiple functions with same name
    }
    Err(e) => return Err(e.into()),
}
```

## Language-Specific Notes

### Rust

- Trait implementations require matching trait changes
- Generic parameters are preserved
- Lifetime annotations are maintained

### TypeScript

- Optional parameters use `?` syntax
- Destructured parameters are handled
- Overloads need manual attention

### Python

- Default values must be valid Python expressions
- `*args` and `**kwargs` are preserved
- Decorators are maintained

### Go

- Named return values are supported
- Multiple return values are handled
- Interface implementations need matching changes

## Best Practices

1. **Preview changes** - Always use `.dry_run()` first
2. **Start with additions** - Add parameters before removing old ones
3. **Provide meaningful defaults** - Help callers migrate gradually
4. **Update tests** - Test call sites may need attention
5. **Consider deprecation** - Mark old signature as deprecated first

## See Also

- [Safe Delete](./safe-delete.md) - Remove unused parameters
- [Extract](./extract.md) - Create functions with proper signatures
