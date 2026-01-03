# Extract Operations

Extract operations create new named entities (functions, variables, constants) from existing code, making it more modular and reusable.

## ExtractFunction

Extract a block of code into a new function.

### Basic Usage

```rust
use refactor::prelude::*;

// Extract lines 10-20 into a new function
let result = ExtractFunction::new("calculate_total")
    .from_file("src/checkout.rs")
    .range(Range::new(
        Position::new(10, 0),
        Position::new(20, 0)
    ))
    .execute()?;

println!("Created function: {}", result.function_name);
```

### Configuration

```rust
ExtractFunction::new("process_items")
    .from_file("src/processor.rs")
    .range(selection)
    // Set visibility
    .visibility(Visibility::Public)  // pub fn
    // .visibility(Visibility::Crate)  // pub(crate) fn
    // .visibility(Visibility::Private)  // fn (default)

    // Control parameter inference
    .parameter_strategy(ParameterStrategy::Infer)  // Auto-detect (default)
    // .parameter_strategy(ParameterStrategy::Explicit(params))  // Specify manually

    // Add documentation
    .with_doc("Processes items and returns the result")

    // Dry run first
    .dry_run()
    .execute()?;
```

### Parameter Inference

The extractor analyzes the selected code to determine:

1. **Parameters** - Variables used but not defined in the selection
2. **Return value** - Values computed and used after the selection
3. **Mutability** - Whether parameters need `&mut`

```rust
// Before extraction:
fn main() {
    let items = vec![1, 2, 3];
    let multiplier = 2;

    // --- Selection start ---
    let mut sum = 0;
    for item in &items {
        sum += item * multiplier;
    }
    // --- Selection end ---

    println!("Sum: {}", sum);
}

// After extraction:
fn calculate_sum(items: &[i32], multiplier: i32) -> i32 {
    let mut sum = 0;
    for item in items {
        sum += item * multiplier;
    }
    sum
}

fn main() {
    let items = vec![1, 2, 3];
    let multiplier = 2;
    let sum = calculate_sum(&items, multiplier);
    println!("Sum: {}", sum);
}
```

### Validation

The extractor validates that:

- Selection contains complete statements
- No partial expressions are selected
- Control flow (return, break, continue) can be handled
- References to local variables are resolvable

```rust
let validation = ExtractFunction::new("helper")
    .from_file("src/main.rs")
    .range(selection)
    .validate()?;

match validation {
    ValidationResult::Valid => {
        println!("Ready to extract");
    }
    ValidationResult::Warning(msg) => {
        println!("Warning: {}", msg);
        // e.g., "Selection contains early return - will be converted to Result"
    }
    ValidationResult::Invalid(msg) => {
        println!("Cannot extract: {}", msg);
        // e.g., "Selection contains partial expression"
    }
}
```

## ExtractVariable

Extract an expression into a named variable.

### Basic Usage

```rust
use refactor::prelude::*;

// Extract expression at position into a variable
let result = ExtractVariable::new("tax_rate")
    .from_file("src/pricing.rs")
    .position(Position::new(15, 20))  // Position within expression
    .execute()?;
```

### Replace All Occurrences

```rust
// Find and replace all identical expressions
ExtractVariable::new("discount_factor")
    .from_file("src/pricing.rs")
    .position(Position::new(15, 20))
    .replace_all_occurrences(true)  // Replace all 5 occurrences
    .execute()?;
```

### Example

```rust
// Before:
fn calculate_price(base: f64, quantity: i32) -> f64 {
    let subtotal = base * quantity as f64;
    let tax = subtotal * 0.08;  // <- position here
    let shipping = if subtotal * 0.08 > 10.0 { 0.0 } else { 5.0 };
    subtotal + subtotal * 0.08 + shipping
}

// After (with replace_all_occurrences):
fn calculate_price(base: f64, quantity: i32) -> f64 {
    let subtotal = base * quantity as f64;
    let tax_amount = subtotal * 0.08;
    let tax = tax_amount;
    let shipping = if tax_amount > 10.0 { 0.0 } else { 5.0 };
    subtotal + tax_amount + shipping
}
```

## ExtractConstant

Extract a literal value into a named constant.

### Basic Usage

```rust
use refactor::prelude::*;

// Extract magic number into a constant
ExtractConstant::new("MAX_RETRIES")
    .from_file("src/client.rs")
    .position(Position::new(8, 25))
    .execute()?;
```

### Configuration

```rust
ExtractConstant::new("DEFAULT_TIMEOUT_SECS")
    .from_file("src/config.rs")
    .position(Position::new(12, 30))
    // Place at module level (default) or in impl block
    .placement(ConstantPlacement::Module)
    // Set visibility
    .visibility(Visibility::Crate)
    // Replace all occurrences of this literal
    .replace_all_occurrences(true)
    .execute()?;
```

### Example

```rust
// Before:
fn connect() -> Result<Connection> {
    let timeout = Duration::from_secs(30);  // <- position here
    let retries = 3;

    for _ in 0..retries {
        match try_connect(Duration::from_secs(30)) {
            Ok(conn) => return Ok(conn),
            Err(_) => sleep(Duration::from_secs(30)),
        }
    }
    Err(Error::Timeout)
}

// After (with replace_all_occurrences):
const DEFAULT_TIMEOUT_SECS: u64 = 30;

fn connect() -> Result<Connection> {
    let timeout = Duration::from_secs(DEFAULT_TIMEOUT_SECS);
    let retries = 3;

    for _ in 0..retries {
        match try_connect(Duration::from_secs(DEFAULT_TIMEOUT_SECS)) {
            Ok(conn) => return Ok(conn),
            Err(_) => sleep(Duration::from_secs(DEFAULT_TIMEOUT_SECS)),
        }
    }
    Err(Error::Timeout)
}
```

## Language Support

Extract operations support:

| Language | Function | Variable | Constant |
|----------|----------|----------|----------|
| Rust     | Yes      | Yes      | Yes      |
| TypeScript | Yes    | Yes      | Yes      |
| Python   | Yes      | Yes      | Yes      |
| Go       | Yes      | Yes      | Yes      |
| Java     | Yes      | Yes      | Yes      |
| C#       | Yes      | Yes      | Yes      |
| Ruby     | Yes      | Yes      | Limited  |

## Error Handling

```rust
use refactor::error::RefactorError;

match ExtractFunction::new("helper").from_file("src/main.rs").range(selection).execute() {
    Ok(result) => {
        println!("Created: {}", result.function_name);
        println!("Diff:\n{}", result.diff());
    }
    Err(RefactorError::InvalidSelection(msg)) => {
        println!("Invalid selection: {}", msg);
    }
    Err(RefactorError::NameConflict(name)) => {
        println!("Name '{}' already exists in scope", name);
    }
    Err(e) => return Err(e.into()),
}
```

## Best Practices

1. **Use meaningful names** - Choose names that describe what the extracted code does
2. **Start small** - Extract simple, self-contained code first
3. **Preview changes** - Always use `.dry_run()` before applying
4. **Check parameters** - Review inferred parameters for correctness
5. **Consider visibility** - Use the minimum visibility needed

## See Also

- [Inline](./inline.md) - The reverse operation
- [Move](./move.md) - Move extracted code to other files
- [Scope Analysis](../scope/README.md) - How variables are tracked
