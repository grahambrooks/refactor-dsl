//! Example: LSP-based rename refactoring
//!
//! This example demonstrates how to use the LSP module to perform
//! semantic rename operations across a codebase using any language
//! server that supports the rename capability.
//!
//! ## Prerequisites
//!
//! You need an LSP server installed for your language:
//! - Rust: `rust-analyzer`
//! - TypeScript/JavaScript: `typescript-language-server`
//! - Python: `pyright-langserver` or `pylsp`
//! - Go: `gopls`
//! - C/C++: `clangd`
//!
//! ## Usage
//!
//! ```bash
//! # Run with a test project
//! cargo run --example lsp_rename -- /path/to/project src/main.rs old_name new_name
//! ```

use refactor_dsl::lsp::{LspRegistry, LspRename, Position};
use refactor_dsl::error::Result;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("LSP Rename Example");
        println!("==================\n");

        // Show available LSP servers
        show_available_servers();

        // Show usage examples
        show_usage_examples();

        // Run a demo if no arguments
        println!("\n=== Demo: Simulated Rename ===\n");
        demo_rename_workflow()?;

        return Ok(());
    }

    // Parse command line arguments
    match args.len() {
        5 => {
            // Full rename: project_path file_path old_name new_name
            let project_path = PathBuf::from(&args[1]);
            let file_path = project_path.join(&args[2]);
            let old_name = &args[3];
            let new_name = &args[4];

            perform_rename(&file_path, old_name, new_name, true)?;
        }
        4 => {
            // Rename with position: file_path line:col new_name
            let file_path = PathBuf::from(&args[1]);
            let position = parse_position(&args[2])?;
            let new_name = &args[3];

            perform_rename_at_position(&file_path, position, new_name, true)?;
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  {} <project_path> <relative_file> <old_name> <new_name>", args[0]);
            eprintln!("  {} <file_path> <line:col> <new_name>", args[0]);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Shows all available LSP server configurations.
fn show_available_servers() {
    println!("Available LSP Servers:");
    println!("----------------------");

    let registry = LspRegistry::new();
    for server in registry.all() {
        println!("  {} ({})", server.name, server.command);
        println!("    Extensions: {}", server.extensions.join(", "));
        println!("    Root markers: {}", server.root_markers.join(", "));
        println!();
    }
}

/// Shows usage examples for different scenarios.
fn show_usage_examples() {
    println!("Usage Examples:");
    println!("---------------\n");

    println!("1. Rename a Rust function:");
    println!("   ```rust");
    println!("   use refactor_dsl::lsp::LspRename;");
    println!();
    println!("   let result = LspRename::find_symbol(");
    println!("       \"src/lib.rs\",");
    println!("       \"old_function\",");
    println!("       \"new_function\",");
    println!("   )?");
    println!("   .dry_run()");
    println!("   .execute()?;");
    println!();
    println!("   println!(\"Changes: {{}}\", result.diff()?);");
    println!("   ```\n");

    println!("2. Rename at a specific position:");
    println!("   ```rust");
    println!("   let result = LspRename::new(\"src/main.rs\", 10, 4, \"new_name\")");
    println!("       .execute()?;");
    println!("   ```\n");

    println!("3. Rename with custom LSP server:");
    println!("   ```rust");
    println!("   let config = LspServerConfig::new(\"custom-lsp\", \"my-lsp-server\")");
    println!("       .args([\"--stdio\"])");
    println!("       .extensions([\"xyz\"]);");
    println!();
    println!("   let result = LspRename::new(\"file.xyz\", 0, 0, \"new_name\")");
    println!("       .server(config)");
    println!("       .execute()?;");
    println!("   ```\n");

    println!("4. Preview changes without applying:");
    println!("   ```rust");
    println!("   let result = LspRename::find_symbol(\"src/lib.rs\", \"foo\", \"bar\")?");
    println!("       .dry_run()");
    println!("       .execute()?;");
    println!();
    println!("   if !result.is_empty() {{");
    println!("       println!(\"Would modify {{}} files:\", result.file_count());");
    println!("       println!(\"{{}}\", result.diff()?);");
    println!("   }}");
    println!("   ```");
}

/// Demonstrates the rename workflow without an actual LSP server.
fn demo_rename_workflow() -> Result<()> {
    println!("Simulating rename of 'calculate_total' -> 'compute_sum'\n");

    // Show what the workflow looks like
    println!("Step 1: Find symbol in source file");
    println!("  File: src/billing.rs");
    println!("  Symbol: calculate_total");
    println!("  Position: line 15, column 4\n");

    println!("Step 2: Start LSP server (rust-analyzer)");
    println!("  Command: rust-analyzer");
    println!("  Root: /project\n");

    println!("Step 3: Initialize LSP connection");
    println!("  -> initialize request");
    println!("  <- capabilities response");
    println!("  -> initialized notification\n");

    println!("Step 4: Open document");
    println!("  -> textDocument/didOpen\n");

    println!("Step 5: Request rename");
    println!("  -> textDocument/rename");
    println!("  <- WorkspaceEdit with changes\n");

    println!("Step 6: Apply changes");
    println!("  Modified files:");
    println!("    - src/billing.rs (3 occurrences)");
    println!("    - src/main.rs (1 occurrence)");
    println!("    - src/tests/billing_test.rs (2 occurrences)\n");

    println!("Step 7: Shutdown LSP server");
    println!("  -> shutdown request");
    println!("  -> exit notification\n");

    // Show example diff output
    println!("Example diff output:");
    println!("--------------------");
    println!("--- a/src/billing.rs");
    println!("+++ b/src/billing.rs");
    println!("@@ -15,7 +15,7 @@");
    println!("-pub fn calculate_total(items: &[Item]) -> f64 {{");
    println!("+pub fn compute_sum(items: &[Item]) -> f64 {{");
    println!("     items.iter().map(|i| i.price).sum()");
    println!(" }}");
    println!();
    println!("--- a/src/main.rs");
    println!("+++ b/src/main.rs");
    println!("@@ -23,7 +23,7 @@");
    println!("-    let total = billing::calculate_total(&cart);");
    println!("+    let total = billing::compute_sum(&cart);");
    println!("     println!(\"Total: ${{}}\", total);");

    Ok(())
}

/// Performs a rename by finding the symbol first.
fn perform_rename(file_path: &PathBuf, old_name: &str, new_name: &str, dry_run: bool) -> Result<()> {
    println!("Renaming '{}' -> '{}' in {}", old_name, new_name, file_path.display());

    let mut rename = LspRename::find_symbol(file_path, old_name, new_name)?;

    if dry_run {
        rename = rename.dry_run();
    }

    let result = rename.execute()?;

    if result.is_empty() {
        println!("No changes needed or symbol not found.");
    } else {
        println!("\nChanges ({} files, {} edits):", result.file_count(), result.edit_count());
        println!("{}", result.diff()?);

        if dry_run {
            println!("\n(dry run - no changes applied)");
        } else {
            println!("\nChanges applied successfully!");
        }
    }

    Ok(())
}

/// Performs a rename at a specific position.
fn perform_rename_at_position(
    file_path: &PathBuf,
    position: Position,
    new_name: &str,
    dry_run: bool,
) -> Result<()> {
    println!(
        "Renaming symbol at {}:{}:{} -> '{}'",
        file_path.display(),
        position.line + 1,
        position.character + 1,
        new_name
    );

    let mut rename = LspRename::new(file_path, position.line, position.character, new_name);

    if dry_run {
        rename = rename.dry_run();
    }

    let result = rename.execute()?;

    if result.is_empty() {
        println!("No changes needed or symbol not found.");
    } else {
        println!("\nChanges ({} files, {} edits):", result.file_count(), result.edit_count());
        println!("{}", result.diff()?);

        if dry_run {
            println!("\n(dry run - no changes applied)");
        } else {
            println!("\nChanges applied successfully!");
        }
    }

    Ok(())
}

/// Parses a position string in the format "line:col" (1-indexed).
fn parse_position(s: &str) -> Result<Position> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 2 {
        return Err(refactor_dsl::error::RefactorError::InvalidConfig(
            "Position must be in format 'line:col'".to_string(),
        ));
    }

    let line: u32 = parts[0].parse().map_err(|_| {
        refactor_dsl::error::RefactorError::InvalidConfig("Invalid line number".to_string())
    })?;

    let col: u32 = parts[1].parse().map_err(|_| {
        refactor_dsl::error::RefactorError::InvalidConfig("Invalid column number".to_string())
    })?;

    // Convert from 1-indexed to 0-indexed
    Ok(Position::new(line.saturating_sub(1), col.saturating_sub(1)))
}
