# Language Support

Refactor DSL provides multi-language support through tree-sitter parsers, enabling AST-based matching and transforms across different programming languages.

## Built-in Languages

| Language | Extensions | Tree-sitter Grammar |
|----------|------------|---------------------|
| Rust | `.rs` | `tree-sitter-rust` |
| TypeScript | `.ts`, `.tsx`, `.js`, `.jsx` | `tree-sitter-typescript` |
| Python | `.py`, `.pyi` | `tree-sitter-python` |

## Language Registry

The `LanguageRegistry` manages available languages:

```rust
use refactor_dsl::prelude::*;

let registry = LanguageRegistry::new();

// Find by extension
let rust = registry.by_extension("rs");

// Find by name
let python = registry.by_name("python");

// Detect from file path
let lang = registry.detect(Path::new("src/main.rs"));

// List all languages
for lang in registry.all() {
    println!("{}: {:?}", lang.name(), lang.extensions());
}
```

## Using Languages Directly

Each language implements the `Language` trait:

```rust
use refactor_dsl::lang::{Rust, TypeScript, Python};

// Parse source code
let tree = Rust.parse("fn main() {}")?;

// Create a query
let query = Rust.query("(function_item name: (identifier) @fn)")?;

// Check extension
assert!(Rust.matches_extension("rs"));
assert!(TypeScript.matches_extension("tsx"));
```

## Language Trait

```rust
pub trait Language: Send + Sync {
    /// Language name (lowercase)
    fn name(&self) -> &'static str;

    /// File extensions handled
    fn extensions(&self) -> &[&'static str];

    /// Tree-sitter grammar
    fn grammar(&self) -> tree_sitter::Language;

    /// Parse source into AST
    fn parse(&self, source: &str) -> Result<Tree>;

    /// Create a tree-sitter query
    fn query(&self, pattern: &str) -> Result<Query>;

    /// Check if extension matches
    fn matches_extension(&self, ext: &str) -> bool;
}
```

## Language-Specific Examples

### Rust

```rust
use refactor_dsl::lang::Rust;

let source = r#"
fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}
"#;

// Parse
let tree = Rust.parse(source)?;
assert!(!tree.root_node().has_error());

// Query for functions
let matcher = AstMatcher::new()
    .query("(function_item name: (identifier) @fn)");
let matches = matcher.find_matches(source, &Rust)?;
```

Common Rust queries:

```rust
// Functions
"(function_item name: (identifier) @fn)"

// Structs
"(struct_item name: (type_identifier) @struct)"

// Enums
"(enum_item name: (type_identifier) @enum)"

// Impl blocks
"(impl_item type: (type_identifier) @impl)"

// Use statements
"(use_declaration) @use"

// Macro invocations
"(macro_invocation macro: (identifier) @macro)"
```

### TypeScript

```rust
use refactor_dsl::lang::TypeScript;

let source = r#"
function greet(name: string): string {
    return `Hello, ${name}!`;
}

const arrow = (x: number) => x * 2;
"#;

let tree = TypeScript.parse(source)?;

let matcher = AstMatcher::new()
    .query("(function_declaration name: (identifier) @fn)");
let matches = matcher.find_matches(source, &TypeScript)?;
```

Common TypeScript queries:

```rust
// Functions
"(function_declaration name: (identifier) @fn)"

// Arrow functions
"(arrow_function) @arrow"

// Classes
"(class_declaration name: (type_identifier) @class)"

// Interfaces
"(interface_declaration name: (type_identifier) @interface)"

// Type aliases
"(type_alias_declaration name: (type_identifier) @type)"

// Imports
"(import_statement) @import"
```

### Python

```rust
use refactor_dsl::lang::Python;

let source = r#"
def greet(name: str) -> str:
    return f"Hello, {name}!"

class Greeter:
    def say_hello(self):
        print("Hello!")
"#;

let tree = Python.parse(source)?;

let matcher = AstMatcher::new()
    .query("(function_definition name: (identifier) @fn)");
let matches = matcher.find_matches(source, &Python)?;
```

Common Python queries:

```rust
// Functions
"(function_definition name: (identifier) @fn)"

// Classes
"(class_definition name: (identifier) @class)"

// Methods (inside class)
"(class_definition body: (block (function_definition name: (identifier) @method)))"

// Imports
"(import_statement) @import"
"(import_from_statement) @from_import"

// Decorators
"(decorated_definition decorator: (decorator) @decorator)"
```

## Adding Custom Languages

To add support for a new language:

1. Add the tree-sitter grammar dependency to `Cargo.toml`
2. Implement the `Language` trait
3. Register with `LanguageRegistry`

```rust
use refactor_dsl::lang::Language;
use tree_sitter::Language as TsLanguage;

// Add to Cargo.toml:
// tree-sitter-go = "0.20"

pub struct Go;

impl Language for Go {
    fn name(&self) -> &'static str {
        "go"
    }

    fn extensions(&self) -> &[&'static str] {
        &["go"]
    }

    fn grammar(&self) -> TsLanguage {
        tree_sitter_go::LANGUAGE.into()
    }
}

// Register
let mut registry = LanguageRegistry::new();
registry.register(Box::new(Go));
```

## Language Detection

Automatic language detection from file paths:

```rust
let registry = LanguageRegistry::new();

// Detects by extension
let lang = registry.detect(Path::new("src/main.rs"));
assert_eq!(lang.unwrap().name(), "rust");

let lang = registry.detect(Path::new("app/index.tsx"));
assert_eq!(lang.unwrap().name(), "typescript");

// No extension or unknown
let lang = registry.detect(Path::new("Makefile"));
assert!(lang.is_none());
```

## Integration with Refactor

Language detection happens automatically in the refactor pipeline:

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .files(|f| f.extension("rs"))  // Language inferred from extension
        .ast(|a| a.query("(function_item @fn)")))  // Rust query
    .transform(/* ... */)
    .apply()?;
```

For explicit language specification:

```rust
let matcher = AstMatcher::new()
    .query("(function_item @fn)");

// Explicit language
let matches = matcher.find_matches(source, &Rust)?;

// Or use registry
let registry = LanguageRegistry::new();
let lang = registry.by_name("rust").unwrap();
let matches = matcher.find_matches(source, lang)?;
```
