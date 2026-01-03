# Language Support

Refactor DSL provides multi-language support through tree-sitter parsers, enabling AST-based matching and transforms
across different programming languages.

## Built-in Languages

| Language   | Extensions                   | Tree-sitter Grammar      | LSP Server                 |
|------------|------------------------------|--------------------------|----------------------------|
| Rust       | `.rs`                        | `tree-sitter-rust`       | rust-analyzer              |
| TypeScript | `.ts`, `.tsx`, `.js`, `.jsx` | `tree-sitter-typescript` | typescript-language-server |
| Python     | `.py`, `.pyi`                | `tree-sitter-python`     | pyright                    |
| Go         | `.go`                        | `tree-sitter-go`         | gopls                      |
| Java       | `.java`                      | `tree-sitter-java`       | jdtls                      |
| C#         | `.cs`                        | `tree-sitter-c-sharp`    | omnisharp                  |
| Ruby       | `.rb`, `.rake`, `.gemspec`   | `tree-sitter-ruby`       | solargraph                 |
| C/C++      | `.c`, `.h`, `.cpp`, `.hpp`   | `tree-sitter-cpp`        | clangd                     |

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
println ! ("{}: {:?}", lang.name(), lang.extensions());
}
```

## Using Languages Directly

Each language implements the `Language` trait:

```rust
use refactor_dsl::lang::{Rust, TypeScript, Python};

// Parse source code
let tree = Rust.parse("fn main() {}") ?;

// Create a query
let query = Rust.query("(function_item name: (identifier) @fn)") ?;

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
let tree = Rust.parse(source) ?;
assert!(!tree.root_node().has_error());

// Query for functions
let matcher = AstMatcher::new()
.query("(function_item name: (identifier) @fn)");
let matches = matcher.find_matches(source, & Rust) ?;
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

let tree = TypeScript.parse(source) ?;

let matcher = AstMatcher::new()
.query("(function_declaration name: (identifier) @fn)");
let matches = matcher.find_matches(source, & TypeScript) ?;
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

let tree = Python.parse(source) ?;

let matcher = AstMatcher::new()
.query("(function_definition name: (identifier) @fn)");
let matches = matcher.find_matches(source, & Python) ?;
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

### Go

```rust
use refactor_dsl::lang::Go;

let source = r#"
package main

import "fmt"

func greet(name string) string {
    return fmt.Sprintf("Hello, %s!", name)
}

type Greeter struct {
    Name string
}

func (g *Greeter) SayHello() {
    fmt.Printf("Hello from %s\n", g.Name)
}
"#;

let tree = Go.parse(source) ?;

let matcher = AstMatcher::new()
.query("(function_declaration name: (identifier) @fn)");
let matches = matcher.find_matches(source, & Go) ?;
```

Common Go queries:

```rust
// Functions
"(function_declaration name: (identifier) @fn)"

// Methods (with receiver)
"(method_declaration
  receiver: (parameter_list) @receiver
  name: (field_identifier) @method)"

// Structs
"(type_declaration (type_spec name: (type_identifier) @struct type: (struct_type)))"

// Interfaces
"(type_declaration (type_spec name: (type_identifier) @interface type: (interface_type)))"

// Imports
"(import_declaration) @import"
"(import_spec path: (interpreted_string_literal) @path)"

// Package declaration
"(package_clause (package_identifier) @package)"
```

### Java

```rust
use refactor_dsl::lang::Java;

let source = r#"
package com.example;

import java.util.List;

public class Greeter {
    private String name;

    public Greeter(String name) {
        this.name = name;
    }

    public String greet() {
        return "Hello, " + name + "!";
    }
}
"#;

let tree = Java.parse(source) ?;

let matcher = AstMatcher::new()
.query("(class_declaration name: (identifier) @class)");
let matches = matcher.find_matches(source, & Java) ?;
```

Common Java queries:

```rust
// Classes
"(class_declaration name: (identifier) @class)"

// Interfaces
"(interface_declaration name: (identifier) @interface)"

// Methods
"(method_declaration name: (identifier) @method)"

// Constructors
"(constructor_declaration name: (identifier) @constructor)"

// Fields
"(field_declaration declarator: (variable_declarator name: (identifier) @field))"

// Imports
"(import_declaration) @import"

// Annotations
"(annotation name: (identifier) @annotation)"

// Package declaration
"(package_declaration) @package"
```

### C#

```rust
use refactor_dsl::lang::CSharp;

let source = r#"
using System;

namespace Example
{
    public class Greeter
    {
        public string Name { get; set; }

        public string Greet()
        {
            return $"Hello, {Name}!";
        }
    }
}
"#;

let tree = CSharp.parse(source) ?;

let matcher = AstMatcher::new()
.query("(class_declaration name: (identifier) @class)");
let matches = matcher.find_matches(source, & CSharp) ?;
```

Common C# queries:

```rust
// Classes
"(class_declaration name: (identifier) @class)"

// Interfaces
"(interface_declaration name: (identifier) @interface)"

// Methods
"(method_declaration name: (identifier) @method)"

// Properties
"(property_declaration name: (identifier) @property)"

// Fields
"(field_declaration (variable_declaration (variable_declarator (identifier) @field)))"

// Using directives
"(using_directive) @using"

// Namespaces
"(namespace_declaration name: (_) @namespace)"

// Records (C# 9+)
"(record_declaration name: (identifier) @record)"
```

### Ruby

```rust
use refactor_dsl::lang::Ruby;

let source = r#"
class Greeter
  attr_accessor :name

  def initialize(name)
    @name = name
  end

  def greet
    "Hello, #{@name}!"
  end
end

module Helpers
  def self.format_name(name)
    name.capitalize
  end
end
"#;

let tree = Ruby.parse(source) ?;

let matcher = AstMatcher::new()
.query("(class name: (constant) @class)");
let matches = matcher.find_matches(source, & Ruby) ?;
```

Common Ruby queries:

```rust
// Classes
"(class name: (constant) @class)"

// Modules
"(module name: (constant) @module)"

// Methods
"(method name: (identifier) @method)"

// Singleton methods (class methods)
"(singleton_method name: (identifier) @class_method)"

// Instance variables
"(instance_variable) @ivar"

// Requires
"(call method: (identifier) @method (#eq? @method \"require\"))"

// attr_accessor, attr_reader, attr_writer
"(call method: (identifier) @accessor (#match? @accessor \"^attr_\"))"

// Blocks
"(block) @block"
"(do_block) @do_block"
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
.matching( | m| m
.files( | f| f.extension("rs"))  // Language inferred from extension
.ast( | a| a.query("(function_item @fn)")))  // Rust query
.transform(/* ... */)
.apply() ?;
```

For explicit language specification:

```rust
let matcher = AstMatcher::new()
.query("(function_item @fn)");

// Explicit language
let matches = matcher.find_matches(source, & Rust) ?;

// Or use registry
let registry = LanguageRegistry::new();
let lang = registry.by_name("rust").unwrap();
let matches = matcher.find_matches(source, lang) ?;
```
