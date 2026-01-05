//! Example: Analyze library changes and generate upgrade codemods.
//!
//! This example demonstrates how to:
//! 1. Compare two versions of a library to detect API changes
//! 2. Generate a codemod from the detected changes
//! 3. Apply the codemod to upgrade client code
//!
//! # Usage
//!
//! ```bash
//! # Analyze Rust library fixtures (dry-run by default)
//! cargo run --example upgrade_analyzer -- rust
//!
//! # Analyze TypeScript library fixtures
//! cargo run --example upgrade_analyzer -- typescript
//!
//! # Analyze and apply changes (modifies files)
//! cargo run --example upgrade_analyzer -- rust --apply
//!
//! # Show detailed diff
//! cargo run --example upgrade_analyzer -- rust --diff
//! ```
//!
//! # Available Languages
//!
//! - rust
//! - typescript
//! - python
//! - go
//! - java
//! - csharp
//! - ruby

#![allow(clippy::print_literal)]

use refactor::analyzer::FileContent;
use refactor::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().collect();

    // Parse arguments
    let language = args.get(1).map(|s| s.as_str()).unwrap_or("rust");
    let apply = args.iter().any(|a| a == "--apply");
    let show_diff = args.iter().any(|a| a == "--diff");
    let help = args.iter().any(|a| a == "--help" || a == "-h");

    if help {
        print_help();
        return Ok(());
    }

    println!("╔═══════════════════════════════════════════════════════════════╗");
    println!("║              Library Upgrade Analyzer Example                 ║");
    println!("╚═══════════════════════════════════════════════════════════════╝\n");

    // Determine fixture paths
    let fixture_dir = Path::new("tests/fixtures").join(format!("{}_library", language));

    if !fixture_dir.exists() {
        println!("Error: Fixture directory not found: {}", fixture_dir.display());
        println!("\nAvailable languages:");
        list_available_fixtures();
        return Ok(());
    }

    let v1_dir = fixture_dir.join("library_v1");
    let v2_dir = fixture_dir.join("library_v2");
    let client_dir = fixture_dir.join("client");

    println!("Language: {}", language.to_uppercase());
    println!("Library v1: {}", v1_dir.display());
    println!("Library v2: {}", v2_dir.display());
    println!("Client: {}", client_dir.display());
    println!();

    // Step 1: Extract APIs from both versions
    println!("═══ Phase 1: API Extraction ═══\n");

    let extension = get_extension_for_language(language);
    let v1_files = collect_source_files(&v1_dir, extension)?;
    let v2_files = collect_source_files(&v2_dir, extension)?;

    println!("Found {} files in v1", v1_files.len());
    println!("Found {} files in v2", v2_files.len());

    let registry = LanguageRegistry::new();
    let extractor = ApiExtractor::with_registry(registry);

    let v1_content = read_files_to_content(&v1_files)?;
    let v2_content = read_files_to_content(&v2_files)?;

    let v1_apis = extractor.extract_all(&v1_content)?;
    let v2_apis = extractor.extract_all(&v2_content)?;

    let v1_count: usize = v1_apis.values().map(|v| v.len()).sum();
    let v2_count: usize = v2_apis.values().map(|v| v.len()).sum();

    println!("Extracted {} API signatures from v1", v1_count);
    println!("Extracted {} API signatures from v2", v2_count);
    println!();

    // Step 2: Detect changes
    println!("═══ Phase 2: Change Detection ═══\n");

    let detector = ChangeDetector::new()
        .rename_threshold(0.7)
        .include_private(false);

    let changes = detector.detect(&v1_apis, &v2_apis);

    if changes.is_empty() {
        println!("No API changes detected between v1 and v2.");
        return Ok(());
    }

    println!("Detected {} API changes:\n", changes.len());

    // Categorize changes
    let mut renames = Vec::new();
    let mut removals = Vec::new();
    let mut signature_changes = Vec::new();
    let mut other_changes = Vec::new();

    for change in &changes {
        match &change.kind {
            ChangeKind::FunctionRenamed { .. } | ChangeKind::TypeRenamed { .. } => {
                renames.push(change);
            }
            ChangeKind::ApiRemoved { .. } => {
                removals.push(change);
            }
            ChangeKind::SignatureChanged { .. }
            | ChangeKind::ParameterAdded { .. }
            | ChangeKind::ParameterRemoved { .. }
            | ChangeKind::ParameterReordered { .. } => {
                signature_changes.push(change);
            }
            _ => {
                other_changes.push(change);
            }
        }
    }

    // Print categorized changes
    if !renames.is_empty() {
        println!("  Renames ({}):", renames.len());
        for change in &renames {
            println!("    • {}", format_change(change));
        }
        println!();
    }

    if !signature_changes.is_empty() {
        println!("  Signature Changes ({}):", signature_changes.len());
        for change in &signature_changes {
            println!("    • {}", format_change(change));
        }
        println!();
    }

    if !removals.is_empty() {
        println!("  Removals ({}):", removals.len());
        for change in &removals {
            println!("    • {}", format_change(change));
        }
        println!();
    }

    if !other_changes.is_empty() {
        println!("  Other Changes ({}):", other_changes.len());
        for change in &other_changes {
            println!("    • {}", format_change(change));
        }
        println!();
    }

    // Step 3: Generate codemod
    println!("═══ Phase 3: Codemod Generation ═══\n");

    let upgrade = UpgradeGenerator::new(
        format!("{}-v1-to-v2", language),
        format!("Upgrade {} library from v1 to v2", language),
    )
    .with_changes(changes)
    .for_extensions(vec![extension.to_string()])
    .generate();

    println!("Generated upgrade: {}", upgrade.name());
    println!("Description: {}", upgrade.description());
    println!();
    println!("Transforms:");
    for transform in &upgrade.transforms {
        let (pattern, replacement) = transform.to_pattern_replacement();
        println!("  • Pattern: {}", truncate(&pattern, 50));
        println!("    Replace: {}", truncate(&replacement, 50));
    }
    println!();

    let auto_count = upgrade.auto_transformable_changes().len();
    let manual_count = upgrade.manual_review_changes().len();
    println!(
        "Summary: {} auto-transformable, {} require manual review",
        auto_count, manual_count
    );
    println!();

    // Step 4: Apply to client (if requested or dry-run)
    println!("═══ Phase 4: Client Migration ═══\n");

    if !client_dir.exists() {
        println!("Client directory not found: {}", client_dir.display());
        return Ok(());
    }

    let result = Refactor::in_repo(&client_dir)
        .matching(|m| m.files(|f| f.extension(extension)))
        .transform(|_| upgrade.transform())
        .dry_run()
        .apply();

    match result {
        Ok(ref refactor_result) => {
            let modified_count = refactor_result
                .changes
                .iter()
                .filter(|c| c.is_modified())
                .count();

            if modified_count == 0 {
                println!("No changes would be made to client code.");
            } else {
                println!("Would modify {} file(s)", modified_count);
                println!();

                if show_diff || !apply {
                    println!("Preview of changes:\n");
                    let diff = refactor_result.diff();
                    if diff.is_empty() {
                        println!("  (no textual changes)");
                    } else {
                        for line in diff.lines().take(50) {
                            println!("  {}", line);
                        }
                        let total_lines = diff.lines().count();
                        if total_lines > 50 {
                            println!("  ... ({} more lines)", total_lines - 50);
                        }
                    }
                    println!();
                }

                if apply {
                    println!("Applying changes...");
                    let apply_result = Refactor::in_repo(&client_dir)
                        .matching(|m| m.files(|f| f.extension(extension)))
                        .transform(|_| upgrade.transform())
                        .apply();

                    match apply_result {
                        Ok(r) => {
                            let applied = r.changes.iter().filter(|c| c.is_modified()).count();
                            println!("✓ Applied changes to {} file(s)", applied);
                        }
                        Err(e) => {
                            println!("✗ Failed to apply changes: {}", e);
                        }
                    }
                } else {
                    println!("Run with --apply to apply these changes.");
                }
            }
        }
        Err(e) => {
            // Handle the "no files matched" case gracefully
            if e.to_string().contains("No files matched") {
                println!("No files matched the filter criteria.");
            } else {
                println!("Error during analysis: {}", e);
            }
        }
    }

    // Print manual review items
    if !upgrade.manual_review_changes().is_empty() {
        println!("\n═══ Manual Review Required ═══\n");
        println!("The following changes cannot be auto-migrated:\n");
        for change in upgrade.manual_review_changes() {
            println!("  ⚠ {}", format_change(change));
            if let Some(notes) = &change.metadata.migration_notes {
                println!("    Note: {}", notes);
            }
        }
    }

    println!("\n╔═══════════════════════════════════════════════════════════════╗");
    println!("║                        Complete!                              ║");
    println!("╚═══════════════════════════════════════════════════════════════╝");

    Ok(())
}

fn print_help() {
    println!(
        r#"
Library Upgrade Analyzer Example

USAGE:
    cargo run --example upgrade_analyzer -- <language> [OPTIONS]

ARGUMENTS:
    <language>    The language fixture to analyze (rust, typescript, python, go, java, csharp, ruby)

OPTIONS:
    --apply       Apply the generated codemod to the client code (modifies files)
    --diff        Show detailed diff of changes
    --help, -h    Show this help message

EXAMPLES:
    # Analyze Rust library changes (dry-run)
    cargo run --example upgrade_analyzer -- rust

    # Analyze TypeScript with detailed diff
    cargo run --example upgrade_analyzer -- typescript --diff

    # Apply changes to Python client
    cargo run --example upgrade_analyzer -- python --apply
"#
    );
}

fn list_available_fixtures() {
    let fixture_base = Path::new("tests/fixtures");
    if let Ok(entries) = fs::read_dir(fixture_base) {
        for entry in entries.filter_map(|e| e.ok()) {
            let name = entry.file_name();
            let name_str = name.to_string_lossy();
            if name_str.ends_with("_library") {
                let lang = name_str.trim_end_matches("_library");
                println!("  - {}", lang);
            }
        }
    }
}

fn get_extension_for_language(language: &str) -> &'static str {
    match language {
        "rust" => "rs",
        "typescript" => "ts",
        "python" => "py",
        "go" => "go",
        "java" => "java",
        "csharp" => "cs",
        "ruby" => "rb",
        _ => "rs",
    }
}

fn collect_source_files(dir: &Path, extension: &str) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();

    fn walk_dir(dir: &Path, extension: &str, files: &mut Vec<PathBuf>) -> std::io::Result<()> {
        if dir.is_dir() {
            for entry in fs::read_dir(dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    walk_dir(&path, extension, files)?;
                } else if path.extension().and_then(|e| e.to_str()) == Some(extension) {
                    files.push(path);
                }
            }
        }
        Ok(())
    }

    walk_dir(dir, extension, &mut files).map_err(|e| {
        RefactorError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to read directory {}: {}", dir.display(), e),
        ))
    })?;

    Ok(files)
}

fn read_files_to_content(files: &[PathBuf]) -> Result<Vec<FileContent>> {
    let mut contents = Vec::new();

    for file in files {
        let content = fs::read_to_string(file).map_err(|e| {
            RefactorError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to read file {}: {}", file.display(), e),
            ))
        })?;

        contents.push(FileContent {
            path: file.clone(),
            content,
        });
    }

    Ok(contents)
}

fn format_change(change: &ApiChange) -> String {
    match &change.kind {
        ChangeKind::FunctionRenamed {
            old_name, new_name, ..
        } => {
            format!("Function renamed: {} -> {}", old_name, new_name)
        }
        ChangeKind::TypeRenamed { old_name, new_name } => {
            format!("Type renamed: {} -> {}", old_name, new_name)
        }
        ChangeKind::ImportRenamed { old_path, new_path } => {
            format!("Import renamed: {} -> {}", old_path, new_path)
        }
        ChangeKind::SignatureChanged { name, .. } => {
            format!("Signature changed: {}", name)
        }
        ChangeKind::ParameterAdded {
            function_name,
            param_name,
            ..
        } => {
            format!("Parameter added to {}: {}", function_name, param_name)
        }
        ChangeKind::ParameterRemoved {
            function_name,
            param_name,
            ..
        } => {
            format!("Parameter removed from {}: {}", function_name, param_name)
        }
        ChangeKind::ParameterReordered { function_name, .. } => {
            format!("Parameters reordered: {}", function_name)
        }
        ChangeKind::ApiRemoved { name, api_type } => {
            format!("{} removed: {}", api_type.name(), name)
        }
        ChangeKind::TypeChanged { name, description } => {
            format!("Type changed: {} - {}", name, description)
        }
        ChangeKind::MethodMoved {
            method_name,
            old_location,
            new_location,
        } => {
            format!(
                "Method moved: {} from {} to {}",
                method_name, old_location, new_location
            )
        }
        ChangeKind::ConstantChanged {
            name,
            old_value,
            new_value,
        } => {
            let old = old_value.as_deref().unwrap_or("?");
            let new = new_value.as_deref().unwrap_or("?");
            format!("Constant changed: {} from {} to {}", name, old, new)
        }
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
