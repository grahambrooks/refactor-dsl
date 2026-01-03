# File Matcher

The `FileMatcher` filters files by extension, path patterns, content, and size.

## Basic Usage

```rust
use refactor::prelude::*;

let matcher = FileMatcher::new()
    .extension("rs")
    .exclude("**/target/**");

let files = matcher.collect(Path::new("./project"))?;
```

## Methods

### Extension Filtering

```rust
// Single extension
.extension("rs")

// Multiple extensions
.extensions(["rs", "toml"])

// Extensions are case-insensitive
.extension("RS")  // matches .rs, .Rs, .RS
```

### Glob Patterns

Include and exclude files using glob patterns:

```rust
// Include only files in src/
.include("**/src/**")

// Exclude test files
.exclude("**/tests/**")
.exclude("**/*_test.rs")

// Multiple patterns
.include("**/src/**")
.include("**/lib/**")
.exclude("**/target/**")
.exclude("**/node_modules/**")
```

Glob pattern syntax:
- `*` - Match any sequence of characters (not including `/`)
- `**` - Match any sequence of characters (including `/`)
- `?` - Match single character
- `[abc]` - Match any character in brackets
- `[!abc]` - Match any character not in brackets

### Content Matching

Match files containing specific regex patterns:

```rust
// Files containing TODO comments
.contains_pattern(r"TODO:")

// Files with deprecated API usage
.contains_pattern(r"deprecated_function\(")

// Files defining public functions
.contains_pattern(r"pub fn \w+")
```

### Name Matching

Match files by name using regex:

```rust
// Files starting with "test_"
.name_matches(r"^test_")

// Files ending with "_spec.rs"
.name_matches(r"_spec\.rs$")

// Main entry points
.name_matches(r"^(main|lib)\.rs$")
```

### Size Filtering

Filter by file size in bytes:

```rust
// Files at least 1KB
.min_size(1024)

// Files no larger than 1MB
.max_size(1024 * 1024)

// Between 1KB and 100KB
.min_size(1024)
.max_size(100 * 1024)
```

## Complete Example

```rust
use refactor::prelude::*;

fn find_large_rust_files_with_todos() -> Result<Vec<PathBuf>> {
    FileMatcher::new()
        .extension("rs")
        .include("**/src/**")
        .exclude("**/target/**")
        .exclude("**/tests/**")
        .contains_pattern(r"TODO|FIXME|HACK")
        .min_size(1024)  // At least 1KB
        .collect(Path::new("./my-project"))
}
```

## Integration with Refactor

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .files(|f| f
            .extension("rs")
            .exclude("**/target/**")
            .contains_pattern("unwrap")))
    .transform(|t| t
        .replace_pattern(r"\.unwrap\(\)", ".expect(\"error\")"))
    .apply()?;
```

## Performance Notes

Filters are applied in order of efficiency:
1. Extension check (fastest)
2. Glob include/exclude
3. Name regex
4. Size check
5. Content regex (slowest - requires reading file)

Place restrictive filters early to skip expensive operations.
