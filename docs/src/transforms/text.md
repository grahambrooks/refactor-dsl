# Text Transforms

Text transforms modify source code using pattern matching and replacement. They work on raw text without understanding code structure.

## Basic Usage

```rust
use refactor_dsl::prelude::*;

let transform = TextTransform::replace(r"\.unwrap\(\)", ".expect(\"error\")");
let result = transform.apply(source, Path::new("file.rs"))?;
```

## Transform Types

### Regex Replacement

Replace text matching a regex pattern:

```rust
// Basic replacement
TextTransform::replace(r"old_api", "new_api")

// With capture groups
TextTransform::replace(r"fn (\w+)", "pub fn $1")

// From pre-compiled regex
let pattern = Regex::new(r"\d+")?;
TextTransform::replace_regex(pattern, "NUM")
```

Regex syntax follows the [regex crate](https://docs.rs/regex/).

**Capture group reference:**
- `$1`, `$2`, etc. - Numbered groups
- `$name` - Named groups (if using `(?P<name>...)`)

### Literal Replacement

Replace exact text without regex interpretation:

```rust
// Safe for special characters
TextTransform::replace_literal("Vec<T>", "Vec<U>")

// Won't interpret .* as regex
TextTransform::replace_literal(".*", "WILDCARD")
```

### Line Operations

#### Prepend to Lines

Add text before matching lines:

```rust
// Add comment before function definitions
TextTransform::prepend_line(r"^\s*fn ", "// TODO: document\n")?

// Add attribute before test functions
TextTransform::prepend_line(r"^\s*fn test_", "#[ignore]\n")?
```

#### Append to Lines

Add text after matching lines:

```rust
// Add comment after statements
TextTransform::append_line(r";\s*$", " // reviewed")?

// Add semicolons to lines ending in certain patterns
TextTransform::append_line(r"\)$", ";")?
```

#### Delete Lines

Remove lines matching a pattern:

```rust
// Remove comment lines
TextTransform::delete_lines(r"^\s*//")?

// Remove empty lines
TextTransform::delete_lines(r"^\s*$")?

// Remove debug statements
TextTransform::delete_lines(r"console\.log\(")?
```

#### Insert After Lines

Insert content after matching lines:

```rust
// Add blank line after imports
TextTransform::insert_after(r"^use ", "")?

// Add attribute after doc comments
TextTransform::insert_after(r"^///", "#[doc(hidden)]")?
```

#### Insert Before Lines

Insert content before matching lines:

```rust
// Add attribute before functions
TextTransform::insert_before(r"^fn ", "#[inline]")?

// Add header comment before module declaration
TextTransform::insert_before(r"^mod ", "// Module:\n")?
```

## Using with TransformBuilder

The `TransformBuilder` provides convenient methods:

```rust
Refactor::in_repo("./project")
    .transform(|t| t
        // Regex replacement
        .replace_pattern(r"old_api\(\)", "new_api()")

        // Literal replacement
        .replace_literal("OldType", "NewType"))
    .apply()?;
```

## Complete Examples

### Replace Deprecated API

```rust
let result = Refactor::in_repo("./project")
    .matching(|m| m.files(|f| f.extension("rs")))
    .transform(|t| t
        .replace_pattern(
            r"deprecated_function\((.*?)\)",
            "new_function($1, Default::default())"
        ))
    .apply()?;
```

### Add Missing Attributes

```rust
use refactor_dsl::transform::TextTransform;

// Add #[derive(Debug)] before struct definitions that don't have it
let transform = TextTransform::insert_before(
    r"^pub struct \w+",
    "#[derive(Debug)]\n"
)?;
```

### Clean Up Comments

```rust
Refactor::in_repo("./project")
    .matching(|m| m.files(|f| f.extension("rs")))
    .transform(|t| t
        .custom(TextTransform::delete_lines(r"^\s*// TODO:").unwrap()))
    .apply()?;
```

### Rename with Context

```rust
// Rename function but preserve formatting
Refactor::in_repo("./project")
    .transform(|t| t
        .replace_pattern(
            r"fn\s+old_function\s*\(",
            "fn new_function("
        )
        .replace_pattern(
            r"old_function\s*\(",
            "new_function("
        ))
    .apply()?;
```

## Error Handling

Invalid regex patterns return errors:

```rust
use refactor_dsl::transform::TextTransform;

// This will panic - invalid regex
// TextTransform::replace(r"[invalid", "replacement")

// Use try methods for fallible patterns
let result = TextTransform::delete_lines(r"[");
assert!(result.is_err());
```

## Performance Tips

1. **Use literal replacement when possible** - Faster than regex
2. **Be specific with patterns** - `^\s*fn ` is faster than `fn `
3. **Order transforms efficiently** - Put common replacements first
4. **Combine patterns when possible** - One regex with `|` vs multiple passes

```rust
// Slower: Multiple passes
.replace_pattern(r"foo", "qux")
.replace_pattern(r"bar", "qux")
.replace_pattern(r"baz", "qux")

// Faster: Single pass
.replace_pattern(r"foo|bar|baz", "qux")
```
