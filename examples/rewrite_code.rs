//! Example: Rewriting code with text and AST transforms
//!
//! This example demonstrates how to use the DSL to perform
//! code transformations including pattern replacement,
//! AST-aware refactoring, and batch symbol renaming.

use refactor_dsl::prelude::*;

fn main() -> Result<()> {
    println!("=== Code Rewriting Examples ===\n");

    // Example 1: Simple text replacement
    println!("Example 1: Text pattern replacement");
    example_text_replacement()?;

    // Example 2: AST-based code matching
    println!("\nExample 2: AST pattern matching");
    example_ast_matching()?;

    // Example 3: Chained transformations
    println!("\nExample 3: Chained transformations");
    example_chained_transforms()?;

    // Example 4: Language-specific refactoring
    println!("\nExample 4: Language-specific refactoring");
    example_language_specific()?;

    Ok(())
}

/// Simple text pattern replacement
fn example_text_replacement() -> Result<()> {
    let source = r#"
fn process_data(data: Vec<u8>) -> Option<String> {
    let parsed = parse(data).unwrap();
    let validated = validate(parsed).unwrap();
    Some(validated.to_string())
}
"#;

    // Replace .unwrap() with .expect("...")
    let transform = TransformBuilder::new()
        .replace_pattern(r"\.unwrap\(\)", r#".expect("TODO: handle error")"#);

    let result = transform.apply(source, std::path::Path::new("example.rs"))?;

    println!("  Before:");
    for line in source.lines().filter(|l| !l.is_empty()) {
        println!("    {}", line);
    }

    println!("\n  After:");
    for line in result.lines().filter(|l| !l.is_empty()) {
        println!("    {}", line);
    }

    Ok(())
}

/// AST-based code matching
fn example_ast_matching() -> Result<()> {
    let rust_source = r#"
fn hello() {
    println!("Hello");
}

fn world() {
    println!("World");
}

pub fn greet(name: &str) {
    println!("Hello, {}!", name);
}
"#;

    // Find all function definitions
    let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn_name)");

    let matches = matcher.find_matches(rust_source, &Rust)?;

    println!("  Found {} functions in Rust code:", matches.len());
    for m in &matches {
        println!(
            "    - '{}' at line {}, column {}",
            m.text,
            m.start_row + 1,
            m.start_col + 1
        );
    }

    // Find function calls
    let call_matcher = AstMatcher::new().query("(call_expression function: (identifier) @fn_call)");

    let calls = call_matcher.find_matches(rust_source, &Rust)?;

    println!("\n  Found {} function calls:", calls.len());
    for m in &calls {
        println!("    - '{}'", m.text);
    }

    Ok(())
}

/// Chained transformations
fn example_chained_transforms() -> Result<()> {
    let source = r#"
use old_crate::OldType;
use old_crate::old_function;

fn main() {
    let x: OldType = old_function();
    let y = x.old_method();
}
"#;

    // Chain multiple transformations
    let transform = TransformBuilder::new()
        // Rename the crate
        .replace_literal("old_crate", "new_crate")
        // Rename the type
        .replace_literal("OldType", "NewType")
        // Rename the function
        .replace_literal("old_function", "new_function")
        // Rename the method
        .replace_literal("old_method", "new_method");

    let result = transform.apply(source, std::path::Path::new("example.rs"))?;

    println!("  Transformation chain:");
    for desc in transform.describe() {
        println!("    - {}", desc);
    }

    println!("\n  Before:");
    for line in source.lines().filter(|l| !l.is_empty()) {
        println!("    {}", line);
    }

    println!("\n  After:");
    for line in result.lines().filter(|l| !l.is_empty()) {
        println!("    {}", line);
    }

    Ok(())
}

/// Language-specific refactoring examples
fn example_language_specific() -> Result<()> {
    // Python example
    let python_source = r#"
def old_function(x):
    return x * 2

class OldClass:
    def old_method(self):
        return old_function(42)
"#;

    let py_matcher = AstMatcher::new().query("(function_definition name: (identifier) @fn)");

    let py_matches = py_matcher.find_matches(python_source, &Python)?;

    println!("  Python functions found:");
    for m in &py_matches {
        println!("    - '{}'", m.text);
    }

    // TypeScript example
    let ts_source = r#"
function fetchData(): Promise<Data> {
    return fetch('/api/data').then(r => r.json());
}

const processData = (data: Data): Result => {
    return transform(data);
};
"#;

    let ts_matcher = AstMatcher::new().query("(function_declaration name: (identifier) @fn)");

    let ts_matches = ts_matcher.find_matches(ts_source, &TypeScript)?;

    println!("\n  TypeScript functions found:");
    for m in &ts_matches {
        println!("    - '{}'", m.text);
    }

    // Arrow functions
    let arrow_matcher = AstMatcher::new().query("(variable_declarator name: (identifier) @name)");

    let arrow_matches = arrow_matcher.find_matches(ts_source, &TypeScript)?;

    println!("\n  TypeScript variable declarations:");
    for m in &arrow_matches {
        println!("    - '{}'", m.text);
    }

    Ok(())
}
