//! Example: Analyze library changes and generate an upgrade codemod.
//!
//! This example demonstrates how to:
//! 1. Analyze API changes between two versions of a library
//! 2. Generate an upgrade that can be applied to dependent projects
//! 3. Save the upgrade configuration to a YAML file
//! 4. Apply the upgrade to multiple repositories
//!
//! Run with:
//! ```sh
//! cargo run --example analyze_library -- /path/to/library v1.0.0 v2.0.0
//! ```

use refactor::analyzer::{LibraryAnalyzer, UpgradeConfig};
use refactor::codemod::Codemod;
use refactor::error::Result;
use std::env;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 4 {
        eprintln!("Usage: {} <library-path> <from-ref> <to-ref> [--apply <projects-dir>]", args[0]);
        eprintln!();
        eprintln!("Examples:");
        eprintln!("  {} ./my-library v1.0.0 v2.0.0", args[0]);
        eprintln!("  {} ./my-library main feature/v2 --apply ./projects", args[0]);
        std::process::exit(1);
    }

    let library_path = PathBuf::from(&args[1]);
    let from_ref = &args[2];
    let to_ref = &args[3];
    let apply_to = args.get(5).map(PathBuf::from);

    println!("Analyzing library: {:?}", library_path);
    println!("From: {} -> To: {}", from_ref, to_ref);
    println!();

    // Create the analyzer
    let analyzer = LibraryAnalyzer::new(&library_path)?
        .for_extensions(vec!["ts", "tsx", "rs", "py"])
        .rename_threshold(0.7);

    // Analyze changes
    println!("Extracting API signatures...");
    let analysis = analyzer.analyze(from_ref, to_ref)?;

    println!("Found {} file changes", analysis.changed_files.len());
    println!("Detected {} API changes:", analysis.changes.len());
    println!("  - Breaking: {}", analysis.breaking_changes().len());
    println!("  - Auto-transformable: {}", analysis.auto_transformable().len());
    println!("  - Requires review: {}", analysis.manual_review().len());
    println!();

    // Generate the upgrade
    let upgrade = analyzer.generate_upgrade(from_ref, to_ref)?;

    // Print the report
    println!("{}", upgrade.report());

    // Save config to YAML
    let config = analyzer.analyze_to_config(from_ref, to_ref)?;
    let config_path = format!(
        "{}-{}-to-{}.yaml",
        library_path.file_name().unwrap().to_str().unwrap(),
        from_ref.replace('/', "-"),
        to_ref.replace('/', "-")
    );
    config.to_yaml(&config_path)?;
    println!("Saved upgrade config to: {}", config_path);

    // Optionally apply to projects
    if let Some(projects_dir) = apply_to {
        println!();
        println!("Applying upgrade to projects in: {:?}", projects_dir);

        let result = Codemod::from_local(&projects_dir)
            .apply(upgrade)
            .dry_run()
            .execute()?;

        println!();
        println!("Dry-run results:");
        println!("  - Repos processed: {}", result.summary.total_repos);
        println!("  - Repos modified: {}", result.summary.modified_repos);
        println!("  - Files modified: {}", result.summary.total_files_modified);

        for repo_result in &result.repo_results {
            if repo_result.files_modified > 0 {
                println!();
                println!("  {} ({} files):", repo_result.name, repo_result.files_modified);
            }
        }
    }

    Ok(())
}

/// Example of loading a saved config and applying it
#[allow(dead_code)]
fn apply_from_config(config_path: &str, projects_dir: &str) -> Result<()> {
    // Load the saved config
    let config = UpgradeConfig::from_yaml(config_path)?;

    println!("Loaded upgrade: {}", config.name);
    println!("Description: {}", config.description);
    if let (Some(from), Some(to)) = (&config.from_version, &config.to_version) {
        println!("Version: {} -> {}", from, to);
    }

    // Convert to an upgrade and apply
    let upgrade = config.to_upgrade();

    let result = Codemod::from_local(projects_dir)
        .apply(upgrade)
        .on_branch("chore/library-upgrade")
        .commit_message("chore: upgrade library")
        .dry_run()
        .execute()?;

    println!("Would modify {} repositories", result.summary.modified_repos);

    Ok(())
}

/// Example of generating Rust source code for a static upgrade
#[allow(dead_code)]
fn generate_rust_upgrade(library_path: &str, from_ref: &str, to_ref: &str) -> Result<()> {
    let analyzer = LibraryAnalyzer::new(library_path)?;
    let upgrade = analyzer.generate_upgrade(from_ref, to_ref)?;

    // Generate Rust source code
    let source = upgrade.to_rust_source("MyLibraryUpgrade");

    println!("Generated Rust upgrade:");
    println!("{}", source);

    Ok(())
}
