//! Example: Restructuring files and directories
//!
//! This example demonstrates how to use the DSL to reorganize
//! project structure, move files, and batch rename operations.

use refactor_dsl::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    println!("=== File Restructuring Examples ===\n");

    // Example 1: Basic file operations
    println!("Example 1: Basic file operations (not executed)");
    example_basic_operations();

    // Example 2: Batch move by pattern
    println!("\nExample 2: Batch move by pattern (not executed)");
    example_batch_move();

    // Example 3: Directory reorganization
    println!("\nExample 3: Directory reorganization (not executed)");
    example_directory_reorg();

    // Example 4: Collect and filter files
    println!("\nExample 4: Collecting files with filters");
    example_collect_files()?;

    Ok(())
}

/// Basic file operations
fn example_basic_operations() {
    let transform = FileTransform::new()
        // Move a single file
        .move_file("src/old_module.rs", "src/new_module.rs")
        // Copy a file
        .copy("config.example.toml", "config.toml")
        // Create a new directory
        .create_dir("src/features")
        // Rename a file
        .rename("README.txt", "README.md")
        // Delete an obsolete file
        .delete("src/deprecated.rs");

    println!("  Planned operations:");
    for desc in transform.describe() {
        println!("    - {}", desc);
    }

    // Note: We're not executing these operations in this example
    // In real usage: transform.execute()?;
}

/// Batch move files matching a pattern
fn example_batch_move() {
    // Imagine we collected these files from a search
    let test_files = vec![
        PathBuf::from("src/utils_test.rs"),
        PathBuf::from("src/parser_test.rs"),
        PathBuf::from("src/lexer_test.rs"),
    ];

    // Move all *_test.rs files to a tests/ directory
    let transform = FileTransform::new().move_matching(test_files, |path| {
        let file_name = path.file_name().unwrap().to_str().unwrap();
        PathBuf::from("tests").join(file_name)
    });

    println!("  Planned operations:");
    for desc in transform.describe() {
        println!("    - {}", desc);
    }
}

/// Directory reorganization example
fn example_directory_reorg() {
    // Reorganize a flat structure into a modular one
    let transform = FileTransform::new()
        // Create new module directories
        .create_dir("src/auth")
        .create_dir("src/api")
        .create_dir("src/db")
        // Move auth-related files
        .move_file("src/login.rs", "src/auth/login.rs")
        .move_file("src/session.rs", "src/auth/session.rs")
        .move_file("src/permissions.rs", "src/auth/permissions.rs")
        // Move API files
        .move_file("src/routes.rs", "src/api/routes.rs")
        .move_file("src/handlers.rs", "src/api/handlers.rs")
        // Move database files
        .move_file("src/models.rs", "src/db/models.rs")
        .move_file("src/queries.rs", "src/db/queries.rs");

    println!("  Planned operations:");
    for desc in transform.describe() {
        println!("    - {}", desc);
    }

    println!("\n  This would transform:");
    println!("    src/");
    println!("      login.rs       -> src/auth/login.rs");
    println!("      session.rs     -> src/auth/session.rs");
    println!("      permissions.rs -> src/auth/permissions.rs");
    println!("      routes.rs      -> src/api/routes.rs");
    println!("      handlers.rs    -> src/api/handlers.rs");
    println!("      models.rs      -> src/db/models.rs");
    println!("      queries.rs     -> src/db/queries.rs");
}

/// Collecting and filtering files
fn example_collect_files() -> Result<()> {
    let current_dir = std::env::current_dir()?;

    // Find all Rust source files, excluding tests and target
    let rust_files = FileMatcher::new()
        .extension("rs")
        .exclude("**/target/**")
        .exclude("**/*_test.rs")
        .exclude("**/tests/**")
        .collect(&current_dir)?;

    println!("  Found {} Rust source files:", rust_files.len());
    for (i, file) in rust_files.iter().take(10).enumerate() {
        println!("    {}. {}", i + 1, file.display());
    }
    if rust_files.len() > 10 {
        println!("    ... and {} more", rust_files.len() - 10);
    }

    // Find files containing a specific pattern
    let files_with_todo = FileMatcher::new()
        .extension("rs")
        .contains_pattern(r"TODO|FIXME")
        .exclude("**/target/**")
        .collect(&current_dir)?;

    println!("\n  Found {} files with TODO/FIXME comments:", files_with_todo.len());
    for file in files_with_todo.iter().take(5) {
        println!("    - {}", file.display());
    }

    Ok(())
}
