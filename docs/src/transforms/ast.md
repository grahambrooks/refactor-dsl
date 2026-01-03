# AST Transforms

AST transforms modify code with awareness of its syntactic structure, using tree-sitter for parsing.

> **Note:** AST transforms in the current version are primarily used for matching and analysis. For structural code modifications, consider using [LSP-based refactoring](../lsp/README.md) which provides semantic understanding.

## Basic Usage

```rust
use refactor::prelude::*;

Refactor::in_repo("./project")
    .transform(|t| t
        .ast(|a| a
            .query("(function_item name: (identifier) @fn)")
            .transform(|node| /* modify */)))
    .apply()?;
```

## AstTransform Builder

The `AstTransform` type allows configuring AST-based transformations:

```rust
let transform = AstTransform::new()
    .query("(function_item name: (identifier) @fn)")
    .language(&Rust);
```

## Query-Based Matching

AST transforms start with a tree-sitter query to identify code patterns:

```rust
// Find all function definitions
.query("(function_item name: (identifier) @fn)")

// Find struct definitions
.query("(struct_item name: (type_identifier) @struct)")

// Find method calls
.query("(call_expression
    function: (field_expression field: (field_identifier) @method))")
```

## Common Patterns

### Finding Functions

```rust
// Rust
"(function_item name: (identifier) @fn)"

// TypeScript
"(function_declaration name: (identifier) @fn)"

// Python
"(function_definition name: (identifier) @fn)"
```

### Finding Classes/Structs

```rust
// Rust
"(struct_item name: (type_identifier) @name)"
"(impl_item type: (type_identifier) @impl_type)"

// TypeScript
"(class_declaration name: (type_identifier) @class)"

// Python
"(class_definition name: (identifier) @class)"
```

### Finding Imports

```rust
// Rust
"(use_declaration argument: (_) @import)"

// TypeScript
"(import_statement source: (string) @source)"

// Python
"(import_statement name: (dotted_name) @import)"
```

## Integration with Text Transforms

AST matching can identify locations for text-based replacement:

```rust
use refactor::prelude::*;

fn rename_function(source: &str, old_name: &str, new_name: &str) -> Result<String> {
    // First, find all occurrences using AST
    let matcher = AstMatcher::new()
        .query("(function_item name: (identifier) @fn)")
        .query("(call_expression function: (identifier) @call)");

    let matches = matcher.find_matches(source, &Rust)?;

    // Filter to our target function
    let target_matches: Vec<_> = matches
        .iter()
        .filter(|m| m.text == old_name)
        .collect();

    // Apply text replacement at those positions
    let mut result = source.to_string();
    for m in target_matches.iter().rev() {
        result.replace_range(m.start_byte..m.end_byte, new_name);
    }

    Ok(result)
}
```

## Combining with File Matching

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .files(|f| f
            .extension("rs")
            .exclude("**/target/**"))
        .ast(|a| a
            .query("(function_item) @fn")))
    .transform(|t| t
        .replace_pattern(r"fn (\w+)", "pub fn $1"))
    .apply()?;
```

## Language Detection

AST transforms require knowing the source language:

```rust
// Explicit language
let transform = AstTransform::new()
    .query("(function_item @fn)")
    .language(&Rust);

// Or detect from file extension
let registry = LanguageRegistry::new();
let lang = registry.detect(Path::new("file.rs")).unwrap();
```

## Limitations

Current AST transform limitations:

1. **Read-only analysis** - AST queries find patterns but don't directly modify the tree
2. **No semantic understanding** - Doesn't understand types, scopes, or references
3. **Single-file scope** - Can't follow imports or understand project structure

For semantic refactoring (cross-file renames, reference updates), use [LSP integration](../lsp/README.md).

## Future Directions

Planned enhancements:

- Direct AST node manipulation
- Structural search and replace
- Code generation from AST templates
- Multi-file AST analysis

## See Also

- [AST Matcher](../matchers/ast.md) - Finding code patterns
- [Tree-sitter Queries](../tree-sitter-queries.md) - Query syntax reference
- [LSP Integration](../lsp/README.md) - Semantic refactoring
