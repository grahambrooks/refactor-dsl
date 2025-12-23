# Tree-sitter Queries

Tree-sitter queries use S-expression syntax to match patterns in parsed syntax trees. This reference covers the query language and common patterns.

## Query Syntax

### Basic Pattern

Match a node by type:

```
(identifier)
```

### Named Children

Access specific child nodes:

```
(function_item name: (identifier))
```

### Captures

Capture matched nodes with `@name`:

```
(function_item name: (identifier) @fn_name)
```

### Anonymous Nodes

Match literal tokens with quotes:

```
(binary_expression operator: "+" @plus)
```

### Wildcards

Match any node type:

```
(call_expression function: (_) @fn)
```

### Alternations

Match multiple patterns:

```
[
  (function_item name: (identifier) @fn)
  (struct_item name: (type_identifier) @fn)
]
```

### Quantifiers

- `?` - Optional (0 or 1)
- `*` - Zero or more
- `+` - One or more

```
(function_item
  (attribute_item)* @attrs
  name: (identifier) @name)
```

### Anchors

- `.` - Anchor to start or end of siblings

```
(block . (expression_statement) @first)  ; First statement
(block (expression_statement) @last .)   ; Last statement
```

### Predicates

#### #eq?

Match exact text:

```
((identifier) @fn
  (#eq? @fn "main"))
```

#### #match?

Match regex pattern:

```
((identifier) @fn
  (#match? @fn "^test_"))
```

#### #not-eq?, #not-match?

Negated versions:

```
((identifier) @fn
  (#not-eq? @fn "main"))
```

## Language Examples

### Rust

```rust
// Function definitions
"(function_item name: (identifier) @fn)"

// Async functions
"(function_item (function_modifiers (async)) name: (identifier) @async_fn)"

// Public functions
"(function_item (visibility_modifier) name: (identifier) @pub_fn)"

// Struct definitions
"(struct_item name: (type_identifier) @struct)"

// Enum definitions
"(enum_item name: (type_identifier) @enum)"

// Impl blocks
"(impl_item type: (type_identifier) @impl_type)"

// Trait definitions
"(trait_item name: (type_identifier) @trait)"

// Use statements
"(use_declaration argument: (_) @import)"

// Macro invocations
"(macro_invocation macro: (identifier) @macro)"

// Method calls
"(call_expression
  function: (field_expression field: (field_identifier) @method))"

// Unsafe blocks
"(unsafe_block) @unsafe"

// Attribute macros
"(attribute_item (attribute) @attr)"

// String literals
"(string_literal) @string"

// Function calls with specific name
"((call_expression
    function: (identifier) @fn)
  (#eq? @fn \"unwrap\"))"
```

### TypeScript/JavaScript

```rust
// Function declarations
"(function_declaration name: (identifier) @fn)"

// Arrow functions
"(arrow_function) @arrow"

// Variable with arrow function
"(variable_declarator
  name: (identifier) @name
  value: (arrow_function))"

// Class declarations
"(class_declaration name: (type_identifier) @class)"

// Method definitions
"(method_definition name: (property_identifier) @method)"

// Interface declarations
"(interface_declaration name: (type_identifier) @interface)"

// Type aliases
"(type_alias_declaration name: (type_identifier) @type)"

// Import statements
"(import_statement) @import"

// Export statements
"(export_statement) @export"

// JSX elements
"(jsx_element
  open_tag: (jsx_opening_element name: (_) @tag))"

// React hooks (functions starting with use)
"((call_expression
    function: (identifier) @hook)
  (#match? @hook \"^use\"))"

// Async functions
"(function_declaration (async) name: (identifier) @async_fn)"
```

### Python

```rust
// Function definitions
"(function_definition name: (identifier) @fn)"

// Class definitions
"(class_definition name: (identifier) @class)"

// Method definitions (in class)
"(class_definition
  body: (block
    (function_definition name: (identifier) @method)))"

// Decorated functions
"(decorated_definition
  definition: (function_definition name: (identifier) @fn))"

// Import statements
"(import_statement) @import"
"(import_from_statement) @from_import"

// Specific imports
"(import_from_statement
  module_name: (dotted_name) @module)"

// Async functions
"(function_definition (async) name: (identifier) @async_fn)"

// Lambda expressions
"(lambda) @lambda"

// Docstrings
"(function_definition
  body: (block . (expression_statement (string)) @docstring))"

// f-strings
"(string (interpolation) @fstring)"

// Type annotations
"(type) @type_annotation"
```

## Query Debugging

### Using tree-sitter CLI

```bash
# Install tree-sitter CLI
npm install -g tree-sitter-cli

# Parse a file and see the tree
tree-sitter parse file.rs

# Run a query against a file
tree-sitter query query.scm file.rs
```

### Online Playground

Use the [tree-sitter playground](https://tree-sitter.github.io/tree-sitter/playground) to interactively develop queries.

### In Refactor DSL

```rust
use refactor_dsl::prelude::*;

// Test query validity
let result = Rust.query("(function_item @fn)");
match result {
    Ok(_) => println!("Valid query"),
    Err(e) => println!("Invalid: {:?}", e),
}

// See what matches
let matches = AstMatcher::new()
    .query("(function_item name: (identifier) @fn)")
    .find_matches(source, &Rust)?;

for m in matches {
    println!("{:?}", m);
}
```

## Common Patterns

### Find All Definitions

```rust
// Functions, structs, enums in Rust
"[
  (function_item name: (identifier) @def)
  (struct_item name: (type_identifier) @def)
  (enum_item name: (type_identifier) @def)
]"
```

### Find Specific Function Calls

```rust
// Calls to deprecated_api
"((call_expression
    function: (identifier) @fn)
  (#eq? @fn \"deprecated_api\"))"

// Method calls to .unwrap()
"((call_expression
    function: (field_expression field: (field_identifier) @method))
  (#eq? @method \"unwrap\"))"
```

### Find TODO Comments

```rust
// Line comments containing TODO
"((line_comment) @comment
  (#match? @comment \"TODO\"))"
```

### Find Test Functions

```rust
// Rust test functions
"(attribute_item (attribute) @attr
  (#eq? @attr \"test\"))
(function_item name: (identifier) @test_fn)"

// Python test functions
"((function_definition name: (identifier) @fn)
  (#match? @fn \"^test_\"))"
```

## Performance Tips

1. **Be specific** - More specific patterns are faster
2. **Use predicates sparingly** - `#eq?` is faster than `#match?`
3. **Capture only what you need** - Extra captures add overhead
4. **Test incrementally** - Start simple, add complexity

## Resources

- [Tree-sitter Documentation](https://tree-sitter.github.io/tree-sitter/)
- [Query Syntax Reference](https://tree-sitter.github.io/tree-sitter/using-parsers#pattern-matching-with-queries)
- Grammar repositories:
  - [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
  - [tree-sitter-typescript](https://github.com/tree-sitter/tree-sitter-typescript)
  - [tree-sitter-python](https://github.com/tree-sitter/tree-sitter-python)
