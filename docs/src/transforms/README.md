# Transforms

Transforms define how to modify matched code. Refactor DSL supports text-based transformations with regex patterns and AST-aware transformations.

## Overview

Transforms are composed using the `TransformBuilder`:

```rust
Refactor::in_repo("./project")
    .matching(/* ... */)
    .transform(|t| t
        .replace_pattern(r"old_api\(\)", "new_api()")
        .replace_literal("OldName", "NewName"))
    .apply()?;
```

## Transform Types

### [Text Transforms](./text.md)

Pattern-based text replacement:

```rust
.transform(|t| t
    // Regex replacement
    .replace_pattern(r"\.unwrap\(\)", ".expect(\"error\")")

    // Literal string replacement
    .replace_literal("old_name", "new_name"))
```

### [AST Transforms](./ast.md)

Structure-aware code transformations:

```rust
.transform(|t| t
    .ast(|a| a
        .query("(function_item name: (identifier) @fn)")
        .transform(|node| /* modify node */)))
```

## Transform Trait

All transforms implement the `Transform` trait:

```rust
pub trait Transform: Send + Sync {
    /// Applies the transformation to source code.
    fn apply(&self, source: &str, path: &Path) -> Result<String>;

    /// Returns a human-readable description.
    fn describe(&self) -> String;
}
```

## Custom Transforms

Implement the `Transform` trait for custom behavior:

```rust
use refactor_dsl::transform::Transform;
use refactor_dsl::error::Result;
use std::path::Path;

struct UppercaseTransform;

impl Transform for UppercaseTransform {
    fn apply(&self, source: &str, _path: &Path) -> Result<String> {
        Ok(source.to_uppercase())
    }

    fn describe(&self) -> String {
        "Convert to uppercase".to_string()
    }
}

// Use with the builder
Refactor::in_repo("./project")
    .transform(|t| t.custom(UppercaseTransform))
    .apply()?;
```

## Transform Composition

Multiple transforms are applied in order:

```rust
.transform(|t| t
    .replace_pattern(r"foo", "bar")       // Applied first
    .replace_pattern(r"bar", "baz")       // Applied second
    .replace_literal("baz", "qux"))       // Applied third

// "foo" -> "bar" -> "baz" -> "qux"
```

## Preview and Description

Get descriptions of configured transforms:

```rust
let builder = TransformBuilder::new()
    .replace_pattern(r"old", "new")
    .replace_literal("foo", "bar");

for desc in builder.describe() {
    println!("{}", desc);
}
// Output:
// Replace pattern 'old' with 'new'
// Replace literal 'foo' with 'bar'
```

## Dry Run

Always preview changes before applying:

```rust
let result = Refactor::in_repo("./project")
    .matching(/* ... */)
    .transform(/* ... */)
    .dry_run()  // Preview only
    .apply()?;

println!("{}", result.diff());
```

## Result Inspection

After applying transforms:

```rust
let result = refactor.apply()?;

// Number of files changed
println!("Modified {} files", result.files_modified());

// Detailed diff
println!("{}", result.diff());

// Colorized diff for terminal
println!("{}", result.colorized_diff());

// Change summary
println!("{}", result.summary);
```

## FileChange Details

Each modified file produces a `FileChange`:

```rust
for change in &result.changes {
    if change.is_modified() {
        println!("Modified: {}", change.path.display());
        // change.original - Original content
        // change.transformed - New content
    }
}
```
