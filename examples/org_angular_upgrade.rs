//! Example: Multi-Repository Angular Upgrade Using the Codemod Builder
//!
//! This example demonstrates the high-level codemod builder pattern for
//! upgrading Angular applications across a GitHub organization.
//!
//! # Usage
//!
//! ```bash
//! # Set your GitHub token
//! export GITHUB_TOKEN=ghp_your_token_here
//!
//! # Run against your organization
//! cargo run --example org_angular_upgrade -- --org your-org-name
//!
//! # Dry-run mode (preview only)
//! cargo run --example org_angular_upgrade -- --org your-org-name --dry-run
//!
//! # Use a local directory instead of GitHub
//! cargo run --example org_angular_upgrade -- --local /path/to/repos
//! ```
//!
//! # What This Example Does
//!
//! 1. Discovers repositories from a GitHub organization (or local directory)
//! 2. Filters to Angular projects (has `angular.json`)
//! 3. Applies Angular v4/v5 → v15+ upgrade transformations
//! 4. Applies RxJS v5 → v6+ migration
//! 5. Creates a feature branch for the changes
//! 6. Commits and pushes the changes
//! 7. Creates pull requests for review

use refactor::prelude::*;
use std::path::PathBuf;

fn main() -> Result<()> {
    let args = parse_args();

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║     Multi-Repository Angular Upgrade with Codemod Builder    ║");
    println!("╚══════════════════════════════════════════════════════════════╝\n");

    if args.dry_run {
        println!("🔍 DRY-RUN MODE: No changes will be made\n");
    }

    let result = if let Some(org) = &args.org {
        run_github_codemod(org, &args)?
    } else if let Some(local) = &args.local {
        run_local_codemod(local, &args)?
    } else {
        // Demo mode - show what would happen
        println!("📖 DEMO MODE: Showing example code and expected output\n");
        demo_mode()?;
        return Ok(());
    };

    print_results(&result);
    Ok(())
}

/// Run codemod against a GitHub organization.
fn run_github_codemod(org: &str, args: &Args) -> Result<CodemodResult> {
    let token = std::env::var("GITHUB_TOKEN").map_err(|_| {
        RefactorError::InvalidConfig(
            "GITHUB_TOKEN environment variable must be set for GitHub operations".into(),
        )
    })?;

    println!("🔗 Connecting to GitHub organization: {}\n", org);

    let mut codemod = Codemod::from_github_org(org, &token)
        // Filter to Angular projects that need upgrading
        .repositories(|r| {
            r.has_file("angular.json")
                .has_file("package.json")
                .not_archived()
                .not_fork()
                .exclude_topic("legacy")
                .exclude_topic("deprecated")
        })
        // Apply upgrade transformations
        .apply(angular_v4v5_upgrade())
        .apply(rxjs_5_to_6_upgrade())
        // Create a feature branch
        .on_branch("chore/angular-v15-upgrade")
        // Set commit message
        .commit_message(
            "chore: upgrade Angular to v15\n\n\
             Automated migration using refactor-dsl codemod builder.\n\n\
             Changes:\n\
             - Migrated @angular/http to @angular/common/http\n\
             - Updated RxJS imports to v6+ barrel imports\n\
             - Removed deprecated .json() calls",
        )
        // Set workspace
        .workspace(args.workspace.clone());

    // Push and create PRs unless in dry-run mode
    if !args.dry_run {
        codemod = codemod.push_branch().create_pr(
            "Upgrade to Angular 15",
            "## Summary\n\n\
                 This PR upgrades the project to Angular 15.\n\n\
                 ### Changes\n\
                 - Migrated from `@angular/http` to `@angular/common/http`\n\
                 - Updated RxJS imports to v6+ barrel imports\n\
                 - Converted to HttpClient (auto JSON parsing)\n\n\
                 ### Testing\n\
                 - [ ] Run `npm test`\n\
                 - [ ] Run `npm run build`\n\
                 - [ ] Verify application works as expected\n\n\
                 ---\n\
                 🤖 Generated with [refactor-dsl](https://github.com/example/refactor-dsl)",
        );
    } else {
        codemod = codemod.dry_run();
    }

    codemod.execute()
}

/// Run codemod against a local directory.
fn run_local_codemod(local_path: &PathBuf, args: &Args) -> Result<CodemodResult> {
    println!(
        "📁 Processing local repositories in: {}\n",
        local_path.display()
    );

    let mut codemod = Codemod::from_local(local_path)
        .repositories(|r| r.has_file("angular.json").has_file("package.json"))
        .apply(angular_v4v5_upgrade())
        .apply(rxjs_5_to_6_upgrade())
        .on_branch("chore/angular-v15-upgrade")
        .commit_message("chore: upgrade Angular to v15")
        .workspace(args.workspace.clone());

    if args.dry_run {
        codemod = codemod.dry_run();
    }

    codemod.execute()
}

/// Demo mode showing example usage.
fn demo_mode() -> Result<()> {
    println!("The codemod builder provides a fluent API for multi-repo refactoring:\n");

    println!("```rust");
    println!("use refactor::prelude::*;");
    println!();
    println!("let result = Codemod::from_github_org(\"acme-corp\", &token)");
    println!("    .repositories(|r| r");
    println!("        .has_file(\"angular.json\")");
    println!("        .not_archived()");
    println!("        .not_fork())");
    println!("    .apply(angular_v4v5_upgrade())");
    println!("    .apply(rxjs_5_to_6_upgrade())");
    println!("    .on_branch(\"chore/angular-v15-upgrade\")");
    println!("    .commit_message(\"chore: upgrade Angular to v15\")");
    println!("    .push_branch()");
    println!("    .create_pr(\"Angular 15 Upgrade\", \"Automated migration...\")");
    println!("    .execute()?;");
    println!("```\n");

    println!("This single builder chain:");
    println!("  1. 🔍 Discovers repos from GitHub org");
    println!("  2. 🎯 Filters to Angular projects");
    println!("  3. 🔄 Applies HttpModule → HttpClientModule migration");
    println!("  4. 🔄 Applies RxJS v5 → v6 migration");
    println!("  5. 🌿 Creates feature branch");
    println!("  6. 💾 Commits changes");
    println!("  7. 📤 Pushes to remote");
    println!("  8. 📝 Creates pull requests\n");

    println!("─────────────────────────────────────────────────────────────────");
    println!("Expected output when run against an organization:\n");

    // Simulated output
    println!("🔗 Connecting to GitHub organization: acme-corp\n");
    println!("📦 Discovered 15 repositories");
    println!("🎯 Filtered to 8 Angular repositories\n");

    let repos = [
        ("frontend-app", 15, "#142"),
        ("admin-dashboard", 11, "#87"),
        ("customer-portal", 22, "#203"),
        ("shared-components", 9, "#45"),
    ];

    for (name, files, pr) in repos {
        println!("📂 Processing {}...", name);
        println!("   ✓ Created branch: chore/angular-v15-upgrade");
        println!("   ✓ Modified {} files", files);
        println!("   ✓ Committed changes");
        println!("   ✓ Pushed to origin");
        println!("   ✓ Created PR {}", pr);
        println!();
    }

    println!("═══════════════════════════════════════════════════════════════");
    println!("                        Summary");
    println!("═══════════════════════════════════════════════════════════════");
    println!("  Repositories processed:  8");
    println!("  Repositories modified:   8");
    println!("  Total files modified:    95");
    println!("  Pull requests created:   8");
    println!("═══════════════════════════════════════════════════════════════");

    Ok(())
}

/// Print the results of a codemod execution.
fn print_results(result: &CodemodResult) {
    println!("\n═══════════════════════════════════════════════════════════════");
    println!("                     Execution Results");
    println!("═══════════════════════════════════════════════════════════════\n");

    for repo_result in &result.repo_results {
        let status_icon = match &repo_result.status {
            refactor::codemod::RepoStatus::Success => "✓",
            refactor::codemod::RepoStatus::NoChanges => "○",
            refactor::codemod::RepoStatus::Skipped(_) => "⊘",
            refactor::codemod::RepoStatus::Failed(_) => "✗",
        };

        println!(
            "  {} {} ({} files)",
            status_icon, repo_result.name, repo_result.files_modified
        );

        if let Some(branch) = &repo_result.branch_created {
            println!("      Branch: {}", branch);
        }

        if let Some(pr_url) = &repo_result.pr_url {
            println!("      PR: {}", pr_url);
        }

        for error in &repo_result.errors {
            println!("      ⚠ {}", error);
        }

        match &repo_result.status {
            refactor::codemod::RepoStatus::Skipped(reason) => {
                println!("      Skipped: {}", reason);
            }
            refactor::codemod::RepoStatus::Failed(reason) => {
                println!("      Failed: {}", reason);
            }
            _ => {}
        }
    }

    println!("\n───────────────────────────────────────────────────────────────");
    println!("                        Summary");
    println!("───────────────────────────────────────────────────────────────");
    println!("  Total repositories:     {}", result.summary.total_repos);
    println!(
        "  Modified:               {}",
        result.summary.modified_repos
    );
    println!("  Skipped:                {}", result.summary.skipped_repos);
    println!("  Failed:                 {}", result.summary.failed_repos);
    println!(
        "  Total files modified:   {}",
        result.summary.total_files_modified
    );
    println!("  PRs created:            {}", result.summary.prs_created);
    println!("───────────────────────────────────────────────────────────────\n");
}

// ─────────────────────────────────────────────────────────────────────────────
// Argument parsing
// ─────────────────────────────────────────────────────────────────────────────

struct Args {
    org: Option<String>,
    local: Option<PathBuf>,
    dry_run: bool,
    workspace: PathBuf,
}

fn parse_args() -> Args {
    let args: Vec<String> = std::env::args().collect();
    let mut org = None;
    let mut local = None;
    let mut dry_run = false;
    let mut workspace = std::env::temp_dir().join("codemod-workspace");

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--org" | "-o" => {
                i += 1;
                if i < args.len() {
                    org = Some(args[i].clone());
                }
            }
            "--local" | "-l" => {
                i += 1;
                if i < args.len() {
                    local = Some(PathBuf::from(&args[i]));
                }
            }
            "--dry-run" | "-n" => {
                dry_run = true;
            }
            "--workspace" | "-w" => {
                i += 1;
                if i < args.len() {
                    workspace = PathBuf::from(&args[i]);
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {}
        }
        i += 1;
    }

    Args {
        org,
        local,
        dry_run,
        workspace,
    }
}

fn print_help() {
    println!("Multi-Repository Angular Upgrade Example");
    println!();
    println!("USAGE:");
    println!("    cargo run --example org_angular_upgrade -- [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --org, -o <ORG>         GitHub organization name");
    println!("    --local, -l <PATH>      Local directory containing repositories");
    println!("    --dry-run, -n           Preview changes without applying");
    println!("    --workspace, -w <PATH>  Workspace directory for cloning repos");
    println!("    --help, -h              Print this help message");
    println!();
    println!("ENVIRONMENT:");
    println!("    GITHUB_TOKEN            Required for GitHub operations");
    println!();
    println!("EXAMPLES:");
    println!("    # Run against a GitHub org");
    println!("    cargo run --example org_angular_upgrade -- --org my-org");
    println!();
    println!("    # Preview changes (dry-run)");
    println!("    cargo run --example org_angular_upgrade -- --org my-org --dry-run");
    println!();
    println!("    # Use local repositories");
    println!("    cargo run --example org_angular_upgrade -- --local /path/to/repos");
    println!();
    println!("    # Demo mode (no arguments)");
    println!("    cargo run --example org_angular_upgrade");
}
