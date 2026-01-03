# Find Dead Code

Dead code analysis identifies unused code that can be safely removed, helping keep your codebase clean and maintainable.

## Overview

`FindDeadCode` detects:

- **Unused functions** - Functions never called
- **Unused types** - Structs, classes, enums with no references
- **Unused imports** - Import statements for unused items
- **Unused variables** - Variables assigned but never read
- **Unreachable code** - Code after unconditional returns
- **Unused parameters** - Function parameters never used

## Basic Usage

```rust
use refactor::prelude::*;

// Find all dead code in a workspace
let report = FindDeadCode::in_workspace("./project")
    .execute()?;

println!("Found {} dead code items:", report.items.len());
for item in &report.items {
    println!("  {} ({}) at {}:{}",
        item.name,
        item.kind,
        item.file.display(),
        item.line);
}
```

## Dead Code Types

### Filter by Type

```rust
let report = FindDeadCode::in_workspace("./project")
    // Select specific types to find
    .include(DeadCodeType::UnusedFunctions)
    .include(DeadCodeType::UnusedImports)
    .include(DeadCodeType::UnusedVariables)
    .execute()?;
```

### All Dead Code Types

```rust
pub enum DeadCodeType {
    UnusedFunctions,    // Functions never called
    UnusedTypes,        // Structs/classes/enums not referenced
    UnusedImports,      // Imported but unused items
    UnusedVariables,    // Assigned but never read
    UnusedParameters,   // Function params never used in body
    UnreachableCode,    // Code after return/break/continue
    UnusedFields,       // Struct fields never accessed
    UnusedConstants,    // Constants never referenced
}
```

### Example Output

```rust
let report = FindDeadCode::in_workspace("./project")
    .include(DeadCodeType::UnusedFunctions)
    .execute()?;

// Report contents:
// DeadCodeItem { name: "old_helper", kind: UnusedFunction, file: "src/utils.rs", line: 45 }
// DeadCodeItem { name: "deprecated_api", kind: UnusedFunction, file: "src/lib.rs", line: 123 }
```

## Scope Control

### Specific File

```rust
let report = FindDeadCode::in_file("src/utils.rs")
    .execute()?;
```

### Specific Directory

```rust
let report = FindDeadCode::in_directory("src/legacy")
    .execute()?;
```

### With Exclusions

```rust
let report = FindDeadCode::in_workspace("./project")
    // Exclude test files
    .exclude("**/tests/**")
    .exclude("**/*_test.rs")
    // Exclude generated code
    .exclude("**/generated/**")
    // Exclude examples
    .exclude("**/examples/**")
    .execute()?;
```

## Analysis Depth

### Cross-File Analysis

By default, analysis considers usages across all files:

```rust
let report = FindDeadCode::in_workspace("./project")
    .cross_file_analysis(true)  // Default
    .execute()?;
```

### Single-File Analysis

Faster but may report false positives:

```rust
let report = FindDeadCode::in_file("src/utils.rs")
    .cross_file_analysis(false)
    .execute()?;
```

## Export Awareness

Control how exports are treated:

```rust
let report = FindDeadCode::in_workspace("./project")
    // Treat all exports as potentially used (library mode)
    .consider_exports_used(true)
    .execute()?;

// Or for applications, exports without usages are dead:
let report = FindDeadCode::in_workspace("./project")
    .consider_exports_used(false)
    .execute()?;
```

## Report Formats

### Text Summary

```rust
let report = FindDeadCode::in_workspace("./project").execute()?;

println!("{}", report.summary());
// Output:
// Dead Code Report
// ================
// Unused functions: 5
// Unused imports: 12
// Unused variables: 3
// Total: 20 items
```

### Detailed Report

```rust
for item in &report.items {
    println!("{}: {} at {}:{}",
        item.kind,
        item.name,
        item.file.display(),
        item.line);

    if let Some(context) = &item.context {
        println!("  {}", context);  // Surrounding code
    }

    if let Some(reason) = &item.reason {
        println!("  Reason: {}", reason);  // Why it's considered dead
    }
}
```

### JSON Export

```rust
let json = report.to_json()?;
std::fs::write("dead-code-report.json", json)?;
```

### SARIF Format (for CI/CD)

```rust
let sarif = report.to_sarif()?;
std::fs::write("dead-code.sarif", sarif)?;
```

## Integration with Safe Delete

Automatically clean up dead code:

```rust
let report = FindDeadCode::in_workspace("./project")
    .include(DeadCodeType::UnusedFunctions)
    .execute()?;

// Preview what would be deleted
for item in &report.items {
    println!("Would delete: {} in {}", item.name, item.file.display());
}

// Delete all dead code (use with caution!)
if user_confirmed() {
    for item in report.items {
        SafeDelete::symbol(&item.name)
            .in_file(&item.file)
            .force(true)  // We know it's unused
            .execute()?;
    }
}
```

## Language-Specific Considerations

### Rust

- Respects `#[allow(dead_code)]` annotations
- Considers trait implementations as used
- Handles conditional compilation (`#[cfg(...)]`)

```rust
let report = FindDeadCode::in_workspace("./project")
    .respect_annotations(true)  // Skip #[allow(dead_code)]
    .execute()?;
```

### TypeScript

- Considers module exports
- Handles type-only imports
- Respects `// @ts-ignore` comments

### Python

- Handles `__all__` exports
- Considers `if __name__ == "__main__"` blocks
- Respects `# noqa` comments

### Go

- Considers exported (capitalized) identifiers
- Handles `init()` functions
- Respects `//nolint` comments

## False Positive Handling

Some code may appear unused but is actually needed:

```rust
let report = FindDeadCode::in_workspace("./project")
    // Skip symbols matching patterns
    .skip_pattern("test_*")
    .skip_pattern("*_benchmark")
    // Skip specific files
    .exclude("**/fixtures/**")
    // Skip items with specific annotations
    .respect_annotations(true)
    .execute()?;
```

### Common False Positives

1. **Reflection/runtime usage** - Code used via strings/macros
2. **FFI exports** - Functions called from C/other languages
3. **Framework callbacks** - Methods called by framework magic
4. **Test fixtures** - Code only used in tests

## CI/CD Integration

### GitHub Actions

```yaml
- name: Check for dead code
  run: |
    refactor dead-code --format sarif > dead-code.sarif

- name: Upload SARIF
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: dead-code.sarif
```

### Pre-commit Hook

```bash
#!/bin/bash
result=$(refactor dead-code --exit-code)
if [ $? -ne 0 ]; then
    echo "Dead code found! Please remove unused code."
    exit 1
fi
```

## Error Handling

```rust
use refactor::error::RefactorError;

match FindDeadCode::in_workspace("./project").execute() {
    Ok(report) => {
        println!("Found {} dead items", report.items.len());
    }
    Err(RefactorError::ParseError { path, message }) => {
        println!("Failed to parse {}: {}", path.display(), message);
    }
    Err(RefactorError::IoError(e)) => {
        println!("IO error: {}", e);
    }
    Err(e) => return Err(e.into()),
}
```

## Language Support

| Language   | Functions | Types | Imports | Variables | Parameters |
|------------|-----------|-------|---------|-----------|------------|
| Rust       | Yes       | Yes   | Yes     | Yes       | Yes        |
| TypeScript | Yes       | Yes   | Yes     | Yes       | Yes        |
| Python     | Yes       | Yes   | Yes     | Yes       | Yes        |
| Go         | Yes       | Yes   | Yes     | Yes       | Yes        |
| Java       | Yes       | Yes   | Yes     | Yes       | Yes        |
| C#         | Yes       | Yes   | Yes     | Yes       | Yes        |
| Ruby       | Yes       | Yes   | Yes     | Yes       | Limited    |

## Best Practices

1. **Run regularly** - Include in CI/CD pipeline
2. **Start conservative** - Use `consider_exports_used(true)` for libraries
3. **Review before deleting** - Some "dead" code may be needed
4. **Exclude tests initially** - Test utilities may seem unused
5. **Handle false positives** - Use annotations or skip patterns

## See Also

- [Safe Delete](./safe-delete.md) - Remove dead code safely
- [Scope Analysis](../scope/README.md) - How usages are tracked
