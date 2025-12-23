# AST Matcher

The `AstMatcher` finds code patterns using tree-sitter queries, enabling language-aware matching that understands code structure.

## Basic Usage

```rust
use refactor_dsl::prelude::*;

let matcher = AstMatcher::new()
    .query("(function_item name: (identifier) @fn_name)");

let matches = matcher.find_matches(
    "fn hello() {} fn world() {}",
    &Rust,
)?;

for m in matches {
    println!("Found function: {}", m.text);
}
```

## Query Syntax

Tree-sitter queries use S-expression syntax. Each query pattern describes a node structure to match.

### Basic Pattern

```
(node_type)
```

Matches any node of that type.

### Named Children

```
(function_item name: (identifier))
```

Matches function items and accesses their `name` child.

### Captures

```
(function_item name: (identifier) @fn_name)
```

The `@fn_name` captures the matched identifier for extraction.

### Multiple Captures

```
(function_item
  name: (identifier) @fn_name
  parameters: (parameters) @params)
```

## Language-Specific Queries

### Rust

```rust
// Function definitions
"(function_item name: (identifier) @fn)"

// Struct definitions
"(struct_item name: (type_identifier) @struct)"

// Method calls
"(call_expression function: (field_expression field: (field_identifier) @method))"

// Use statements
"(use_declaration argument: (scoped_identifier) @import)"
```

### TypeScript

```rust
// Function declarations
"(function_declaration name: (identifier) @fn)"

// Arrow functions
"(arrow_function) @arrow"

// Class declarations
"(class_declaration name: (type_identifier) @class)"

// Method definitions
"(method_definition name: (property_identifier) @method)"
```

### Python

```rust
// Function definitions
"(function_definition name: (identifier) @fn)"

// Class definitions
"(class_definition name: (identifier) @class)"

// Import statements
"(import_statement name: (dotted_name) @import)"

// Function calls
"(call function: (identifier) @call)"
```

## Methods

### Adding Queries

```rust
// Single query
.query("(function_item name: (identifier) @fn)")

// Multiple queries
.query("(function_item name: (identifier) @fn)")
.query("(struct_item name: (type_identifier) @struct)")
```

### Filtering Captures

```rust
// Only get specific captures
.query("(function_item name: (identifier) @fn)")
.query("(parameter pattern: (identifier) @param)")
.capture("fn")  // Only return @fn captures
```

## Match Results

Each match contains:

```rust
pub struct AstMatch {
    pub text: String,       // Matched text
    pub start_byte: usize,  // Byte offset start
    pub end_byte: usize,    // Byte offset end
    pub start_row: usize,   // Line number (0-indexed)
    pub start_col: usize,   // Column (0-indexed)
    pub end_row: usize,
    pub end_col: usize,
    pub capture_name: String,  // The @name used
}
```

## File-Based Matching

```rust
let matcher = AstMatcher::new()
    .query("(function_item name: (identifier) @fn)");

let registry = LanguageRegistry::new();
let matches = matcher.find_matches_in_file(
    Path::new("src/main.rs"),
    &registry,
)?;
```

## Check for Matches

```rust
// Just check if pattern exists
if matcher.has_matches(source, &Rust)? {
    println!("Found matching code");
}
```

## Complete Example

Find all functions that call `unwrap()`:

```rust
use refactor_dsl::prelude::*;

fn find_unwrap_calls(path: &Path) -> Result<Vec<AstMatch>> {
    let source = fs::read_to_string(path)?;

    let matcher = AstMatcher::new()
        .query(r#"
            (call_expression
                function: (field_expression
                    field: (field_identifier) @method)
                (#eq? @method "unwrap"))
        "#);

    matcher.find_matches(&source, &Rust)
}
```

## Integration with Refactor

The AST matcher can be used with the `Refactor` builder, but note that AST matching is primarily for finding patterns, not filtering files in the current implementation:

```rust
// Use FileMatcher with content patterns for filtering
Refactor::in_repo("./project")
    .matching(|m| m
        .files(|f| f
            .extension("rs")
            .contains_pattern(r"\.unwrap\(\)")))
    .transform(/* ... */)
    .apply()?;

// Use AstMatcher separately for precise code analysis
let ast_matcher = AstMatcher::new()
    .query("(call_expression) @call");
```

## Finding Query Patterns

To discover the correct query patterns for your language:

1. Use the tree-sitter CLI: `tree-sitter parse file.rs`
2. Use the [tree-sitter playground](https://tree-sitter.github.io/tree-sitter/playground)
3. Check grammar definitions in tree-sitter-{language} repositories

## Error Handling

Invalid queries return an error:

```rust
let matcher = AstMatcher::new()
    .query("(invalid_node_type @x)");

match matcher.find_matches(source, &Rust) {
    Ok(matches) => { /* ... */ }
    Err(RefactorError::Query(_)) => {
        println!("Invalid query syntax");
    }
    Err(e) => return Err(e),
}
```
