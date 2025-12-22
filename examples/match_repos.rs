//! Example: Matching repositories by various criteria
//!
//! This example demonstrates how to use the DSL to filter repositories
//! based on Git state, file presence, and other criteria.

use refactor_dsl::prelude::*;

fn main() -> Result<()> {
    // Example 1: Match a repository on a specific branch
    println!("=== Example 1: Branch matching ===");
    match_by_branch(".")?;

    // Example 2: Match repositories with specific files
    println!("\n=== Example 2: File presence matching ===");
    match_by_files(".")?;

    // Example 3: Match repositories with recent commits
    println!("\n=== Example 3: Recent commit matching ===");
    match_by_recent_activity(".")?;

    // Example 4: Combine multiple criteria
    println!("\n=== Example 4: Combined criteria ===");
    match_combined(".")?;

    Ok(())
}

/// Match repository by branch name
fn match_by_branch(path: &str) -> Result<()> {
    let matcher = Matcher::new().git(|g| g.branch("main"));

    match matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Repository is on 'main' branch"),
        Ok(false) => println!("  Repository is NOT on 'main' branch"),
        Err(e) => println!("  Error: {}", e),
    }

    // Also try 'master' branch
    let matcher = Matcher::new().git(|g| g.branch("master"));

    match matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Repository is on 'master' branch"),
        Ok(false) => println!("  Repository is NOT on 'master' branch"),
        Err(e) => println!("  Error: {}", e),
    }

    Ok(())
}

/// Match repository by file presence
fn match_by_files(path: &str) -> Result<()> {
    // Check for Cargo.toml (Rust project)
    let rust_matcher = Matcher::new().git(|g| g.has_file("Cargo.toml"));

    match rust_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Found Cargo.toml - this is a Rust project"),
        Ok(false) => println!("  No Cargo.toml found"),
        Err(e) => println!("  Error: {}", e),
    }

    // Check for package.json (Node.js project)
    let node_matcher = Matcher::new().git(|g| g.has_file("package.json"));

    match node_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Found package.json - this is a Node.js project"),
        Ok(false) => println!("  No package.json found"),
        Err(e) => println!("  Error: {}", e),
    }

    // Check for pyproject.toml (Python project)
    let python_matcher = Matcher::new().git(|g| g.has_file("pyproject.toml"));

    match python_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Found pyproject.toml - this is a Python project"),
        Ok(false) => println!("  No pyproject.toml found"),
        Err(e) => println!("  Error: {}", e),
    }

    Ok(())
}

/// Match repository by recent commit activity
fn match_by_recent_activity(path: &str) -> Result<()> {
    // Check for commits in the last 7 days
    let recent_matcher = Matcher::new().git(|g| g.recent_commits(7));

    match recent_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Repository has commits in the last 7 days"),
        Ok(false) => println!("  No commits in the last 7 days"),
        Err(e) => println!("  Error: {}", e),
    }

    // Check for commits in the last 30 days
    let month_matcher = Matcher::new().git(|g| g.recent_commits(30));

    match month_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Repository has commits in the last 30 days"),
        Ok(false) => println!("  No commits in the last 30 days"),
        Err(e) => println!("  Error: {}", e),
    }

    Ok(())
}

/// Match repository with combined criteria
fn match_combined(path: &str) -> Result<()> {
    // Match: main branch + Cargo.toml + recent activity + clean state
    let strict_matcher = Matcher::new().git(|g| {
        g.branch("main")
            .has_file("Cargo.toml")
            .recent_commits(30)
            .clean()
    });

    match strict_matcher.matches_repo(std::path::Path::new(path)) {
        Ok(true) => println!("  Repository matches ALL criteria:"),
        Ok(false) => println!("  Repository does NOT match all criteria"),
        Err(e) => println!("  Error: {}", e),
    }

    println!("    - On 'main' branch");
    println!("    - Has Cargo.toml");
    println!("    - Has commits in last 30 days");
    println!("    - Clean working directory");

    Ok(())
}
