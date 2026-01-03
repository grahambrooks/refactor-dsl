# Refactoring Operations

Refactor DSL provides IDE-like refactoring operations that understand code structure, track references, and update all affected locations automatically.

## Overview

Unlike simple text transforms, refactoring operations:

- **Understand scope** - Track variable bindings and visibility
- **Resolve references** - Find all usages across files
- **Maintain correctness** - Update imports, call sites, and signatures
- **Preview changes** - Dry-run mode shows what will change
- **Validate safety** - Check for conflicts before applying

## Available Operations

### [Extract](./extract.md)

Extract code into new functions, variables, or constants:

```rust
use refactor::prelude::*;

// Extract lines 10-20 into a new function
ExtractFunction::new("calculate_total")
    .from_file("src/checkout.rs")
    .range(Range::new(Position::new(10, 0), Position::new(20, 0)))
    .visibility(Visibility::Public)
    .execute()?;

// Extract repeated expression into a variable
ExtractVariable::new("tax_rate")
    .from_file("src/pricing.rs")
    .position(Position::new(15, 20))
    .replace_all_occurrences(true)
    .execute()?;

// Extract magic number into a named constant
ExtractConstant::new("MAX_RETRIES")
    .from_file("src/client.rs")
    .position(Position::new(8, 25))
    .execute()?;
```

### [Inline](./inline.md)

Inline variables and functions back into their usage sites:

```rust
// Inline a variable (replace usages with its value)
InlineVariable::new("temp")
    .in_file("src/process.rs")
    .position(Position::new(12, 8))
    .execute()?;

// Inline a function (replace calls with function body)
InlineFunction::new("helper")
    .in_file("src/utils.rs")
    .all_call_sites(true)
    .execute()?;
```

### [Move](./move.md)

Move code between files and modules:

```rust
// Move a function to another file
MoveToFile::new("process_data")
    .from_file("src/utils.rs")
    .to_file("src/processors.rs")
    .update_imports(true)
    .execute()?;

// Move between modules with re-exports
MoveBetweenModules::new("DataProcessor")
    .from_module("crate::utils")
    .to_module("crate::processors")
    .add_reexport(true)
    .execute()?;
```

### [Change Signature](./signature.md)

Modify function signatures with call-site updates:

```rust
// Add a new parameter with default value
ChangeSignature::for_function("process")
    .in_file("src/lib.rs")
    .add_parameter("timeout", "Duration", "Duration::from_secs(30)")
    .execute()?;

// Reorder parameters
ChangeSignature::for_function("create_user")
    .in_file("src/users.rs")
    .reorder_parameters(&["name", "email", "role"])
    .execute()?;

// Remove unused parameter
ChangeSignature::for_function("old_api")
    .in_file("src/legacy.rs")
    .remove_parameter("unused_flag")
    .execute()?;
```

### [Safe Delete](./safe-delete.md)

Remove code with usage checking:

```rust
// Delete a function, warning if it has usages
SafeDelete::symbol("unused_helper")
    .in_file("src/utils.rs")
    .check_usages(true)
    .execute()?;

// Delete with cascade (also delete dependent code)
SafeDelete::symbol("deprecated_module")
    .in_file("src/lib.rs")
    .cascade(true)
    .execute()?;
```

### [Find Dead Code](./dead-code.md)

Analyze and report unused code:

```rust
// Find all dead code in workspace
let report = FindDeadCode::in_workspace("./project")
    .include(DeadCodeType::UnusedFunctions)
    .include(DeadCodeType::UnusedImports)
    .include(DeadCodeType::UnreachableCode)
    .execute()?;

// Print the report
for item in &report.items {
    println!("{}: {} at {}:{}",
        item.kind,
        item.name,
        item.file.display(),
        item.line);
}

// Generate JSON report
let json = report.to_json()?;
```

## Refactoring Context

All operations share a common `RefactoringContext` that provides:

```rust
pub struct RefactoringContext {
    pub workspace_root: PathBuf,
    pub target_file: PathBuf,
    pub target_range: Range,
    pub language: Box<dyn Language>,
    pub scope_analyzer: ScopeAnalyzer,
    pub lsp_client: Option<LspClient>,
}
```

## Common Patterns

### Preview Before Apply

Always preview changes first:

```rust
let result = ExtractFunction::new("helper")
    .from_file("src/main.rs")
    .range(selection)
    .dry_run()  // Preview only
    .execute()?;

println!("Preview:\n{}", result.diff());

// If satisfied, apply
if user_confirmed() {
    ExtractFunction::new("helper")
        .from_file("src/main.rs")
        .range(selection)
        .execute()?;  // Actually apply
}
```

### Batch Operations

Apply multiple refactorings in sequence:

```rust
// Find and fix all issues
let dead_code = FindDeadCode::in_workspace("./project")
    .include(DeadCodeType::UnusedFunctions)
    .execute()?;

for item in dead_code.items {
    SafeDelete::symbol(&item.name)
        .in_file(&item.file)
        .execute()?;
}
```

### With LSP Enhancement

Use LSP for better accuracy:

```rust
ExtractFunction::new("process")
    .from_file("src/main.rs")
    .range(selection)
    .use_lsp(true)  // Use LSP for type inference
    .auto_install_lsp(true)  // Install LSP if needed
    .execute()?;
```

## Error Handling

```rust
use refactor::error::RefactorError;

match SafeDelete::symbol("helper").in_file("src/lib.rs").execute() {
    Ok(result) => println!("Deleted successfully"),
    Err(RefactorError::SymbolInUse { usages }) => {
        println!("Cannot delete: {} usages found", usages.len());
        for usage in usages {
            println!("  - {}:{}", usage.file.display(), usage.line);
        }
    }
    Err(RefactorError::SymbolNotFound(name)) => {
        println!("Symbol '{}' not found", name);
    }
    Err(e) => return Err(e.into()),
}
```

## Validation

Operations validate before executing:

```rust
let result = ExtractFunction::new("helper")
    .from_file("src/main.rs")
    .range(selection)
    .validate()?;  // Just validate, don't execute

match result {
    ValidationResult::Valid => println!("Ready to extract"),
    ValidationResult::Warning(msg) => println!("Warning: {}", msg),
    ValidationResult::Invalid(msg) => println!("Cannot extract: {}", msg),
}
```

## See Also

- [Scope Analysis](../scope/README.md) - How bindings and references are tracked
- [LSP Integration](../lsp/README.md) - Enhanced refactoring with LSP
- [Transforms](../transforms/README.md) - Simpler text-based transforms
