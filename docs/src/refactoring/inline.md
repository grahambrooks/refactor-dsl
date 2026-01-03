# Inline Operations

Inline operations replace named entities with their definitions, reversing extraction. This can simplify code by removing unnecessary indirection.

## InlineVariable

Replace a variable with its value at all usage sites.

### Basic Usage

```rust
use refactor::prelude::*;

// Inline a variable
let result = InlineVariable::new("temp")
    .in_file("src/process.rs")
    .position(Position::new(12, 8))  // Position of the variable declaration
    .execute()?;

println!("Inlined {} usages", result.replacements);
```

### Example

```rust
// Before:
fn calculate(x: i32, y: i32) -> i32 {
    let sum = x + y;      // <- inline this
    let doubled = sum * 2;
    doubled + sum
}

// After:
fn calculate(x: i32, y: i32) -> i32 {
    let doubled = (x + y) * 2;
    doubled + (x + y)
}
```

### Configuration

```rust
InlineVariable::new("result")
    .in_file("src/lib.rs")
    .position(Position::new(10, 8))
    // Only inline at specific position (not all usages)
    .single_usage(Position::new(15, 12))
    // Keep the declaration (useful for debugging)
    .keep_declaration(true)
    // Preview first
    .dry_run()
    .execute()?;
```

### When to Inline Variables

Good candidates for inlining:

- Single-use variables that don't add clarity
- Trivial expressions like `let x = y;`
- Variables that make code harder to read

Avoid inlining:

- Variables with meaningful names that document intent
- Complex expressions used multiple times
- Variables that capture expensive computations

## InlineFunction

Replace function calls with the function body.

### Basic Usage

```rust
use refactor::prelude::*;

// Inline all calls to a function
let result = InlineFunction::new("helper")
    .in_file("src/utils.rs")
    .all_call_sites(true)
    .execute()?;

println!("Inlined {} call sites", result.call_sites_inlined);
```

### Single Call Site

```rust
// Inline just one specific call
InlineFunction::new("process")
    .in_file("src/main.rs")
    .call_site(Position::new(25, 10))  // Position of the call
    .execute()?;
```

### Example

```rust
// Before:
fn double(x: i32) -> i32 {
    x * 2
}

fn calculate(value: i32) -> i32 {
    let a = double(value);
    let b = double(a);
    a + b
}

// After inlining `double`:
fn calculate(value: i32) -> i32 {
    let a = value * 2;
    let b = a * 2;
    a + b
}
```

### Configuration

```rust
InlineFunction::new("helper")
    .in_file("src/utils.rs")
    // Inline all call sites
    .all_call_sites(true)
    // Also delete the function definition
    .delete_definition(true)
    // Handle parameter renaming to avoid conflicts
    .rename_parameters(true)
    // Preview first
    .dry_run()
    .execute()?;
```

### Parameter Substitution

Parameters are substituted with their arguments:

```rust
// Before:
fn greet(name: &str, times: i32) {
    for _ in 0..times {
        println!("Hello, {}!", name);
    }
}

fn main() {
    greet("World", 3);
}

// After inlining the call:
fn main() {
    for _ in 0..3 {
        println!("Hello, {}!", "World");
    }
}
```

### Handling Complex Cases

The inliner handles:

- **Early returns** - Converted to conditionals when possible
- **Local variables** - Renamed to avoid conflicts
- **Multiple statements** - Wrapped in blocks if needed
- **Side effects** - Preserved in evaluation order

```rust
// Before:
fn compute(x: i32) -> i32 {
    if x < 0 {
        return 0;  // Early return
    }
    x * 2
}

fn process(value: i32) -> i32 {
    let result = compute(value);
    result + 1
}

// After inlining `compute`:
fn process(value: i32) -> i32 {
    let result = if value < 0 {
        0
    } else {
        value * 2
    };
    result + 1
}
```

## Validation

Both operations validate before executing:

```rust
let validation = InlineVariable::new("temp")
    .in_file("src/main.rs")
    .position(Position::new(10, 8))
    .validate()?;

match validation {
    ValidationResult::Valid => println!("Ready to inline"),
    ValidationResult::Warning(msg) => {
        println!("Warning: {}", msg);
        // e.g., "Variable is used 5 times - consider keeping it"
    }
    ValidationResult::Invalid(msg) => {
        println!("Cannot inline: {}", msg);
        // e.g., "Variable has side effects in initialization"
    }
}
```

## Error Handling

```rust
use refactor::error::RefactorError;

match InlineFunction::new("helper").in_file("src/lib.rs").all_call_sites(true).execute() {
    Ok(result) => {
        println!("Inlined {} calls", result.call_sites_inlined);
    }
    Err(RefactorError::SymbolNotFound(name)) => {
        println!("Function '{}' not found", name);
    }
    Err(RefactorError::CannotInline(reason)) => {
        println!("Cannot inline: {}", reason);
        // e.g., "Function is recursive"
        // e.g., "Function has multiple return points that cannot be merged"
    }
    Err(e) => return Err(e.into()),
}
```

## Language Support

| Language   | Variable | Function |
|------------|----------|----------|
| Rust       | Yes      | Yes      |
| TypeScript | Yes      | Yes      |
| Python     | Yes      | Yes      |
| Go         | Yes      | Yes      |
| Java       | Yes      | Yes      |
| C#         | Yes      | Yes      |
| Ruby       | Yes      | Yes      |

## Best Practices

1. **Start with single usage** - Inline one call site to verify correctness
2. **Check for side effects** - Ensure inlining doesn't change behavior
3. **Consider readability** - Sometimes the named entity is clearer
4. **Preview changes** - Always use `.dry_run()` first
5. **Clean up after** - Remove unused definitions

## See Also

- [Extract](./extract.md) - The reverse operation
- [Safe Delete](./safe-delete.md) - Safely remove unused code
