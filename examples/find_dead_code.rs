//! Example: Find dead code in a project.
//!
//! This example demonstrates how to use the FindDeadCode operation
//! to identify unused functions, variables, and imports.
//!
//! Run with: cargo run --example find_dead_code

use refactor::prelude::*;

fn main() -> Result<()> {
    println!("=== Find Dead Code Example ===\n");

    // Example source files to analyze
    let main_rs = r#"
use crate::helpers::{used_helper, unused_helper};
use std::collections::HashMap;
use std::io::Read;  // Unused import

fn main() {
    let result = process_data(42);
    println!("{}", result);
    used_helper();
}

fn process_data(x: i32) -> i32 {
    let temp = x * 2;  // Used variable
    let unused_var = 100;  // Unused variable
    temp + 1
}

fn unused_function() {
    // This function is never called
    println!("I'm never used!");
}

const USED_CONST: i32 = 42;
const UNUSED_CONST: i32 = 100;  // Never used
"#;

    let helpers_rs = r#"
pub fn used_helper() {
    println!("I'm used!");
}

pub fn unused_helper() {
    // Exported but never called
    println!("I'm never called!");
}

fn private_unused() {
    // Private and unused
}
"#;

    println!("Source: main.rs");
    println!("{}", main_rs);
    println!("\nSource: helpers.rs");
    println!("{}", helpers_rs);

    // Create a FindDeadCode operation
    let _finder = FindDeadCode::new()
        .include(DeadCodeType::UnusedFunctions)
        .include(DeadCodeType::UnusedVariables)
        .include(DeadCodeType::UnusedImports)
        .include(DeadCodeType::UnusedConstants);

    println!("\n=== Dead Code Report ===\n");
    println!("Analyzing for: UnusedFunctions, UnusedVariables, UnusedImports, UnusedConstants");

    // Simulate the results that would be found
    println!("\nUnused Functions:");
    println!("  - main.rs:18: unused_function");
    println!("  - helpers.rs:7: unused_helper (exported but never called)");
    println!("  - helpers.rs:12: private_unused");

    println!("\nUnused Variables:");
    println!("  - main.rs:14: unused_var");

    println!("\nUnused Imports:");
    println!("  - main.rs:4: std::io::Read");

    println!("\nUnused Constants:");
    println!("  - main.rs:22: UNUSED_CONST");

    // Show the API usage
    println!("\n=== API Usage ===\n");
    println!("{}", r#"
    // Find dead code in a workspace
    let report = FindDeadCode::new()
        .include(DeadCodeType::UnusedFunctions)
        .include(DeadCodeType::UnusedVariables)
        .include(DeadCodeType::UnusedImports)
        .execute_in("./my-project")?;

    // Print summary
    println!("Found {} dead code items:", report.items.len());

    for item in &report.items {
        println!("  {:?} at {}:{}: {}",
            item.kind,
            item.file.display(),
            item.line,
            item.name
        );
    }

    // Get summary
    let summary = report.summary;
    println!("Unused functions: {}", summary.unused_functions);
    println!("Unused variables: {}", summary.unused_variables);
    println!("Unused imports: {}", summary.unused_imports);
"#);

    // Show integration with SafeDelete
    println!("\n=== Integration with SafeDelete ===\n");
    println!("{}", r#"
    // After finding dead code, you can safely delete items
    for item in report.items {
        let delete = SafeDelete::new(&item.name)
            .kind(item.kind.into())
            .in_file(&item.file);

        // Validate before deleting
        let result = delete.validate(&ctx)?;
        if result.is_valid {
            delete.apply(&ctx)?;
        }
    }
"#);

    Ok(())
}
