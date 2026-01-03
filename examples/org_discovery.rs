//! Example: Advanced repository discovery and filtering.
//!
//! This example demonstrates how to use the AdvancedRepoFilter
//! to discover and filter repositories in a GitHub organization
//! by dependencies, frameworks, metrics, and languages.
//!
//! Run with: cargo run --example org_discovery

use refactor::prelude::*;

fn main() -> Result<()> {
    println!("=== Organization Repository Discovery ===\n");

    // Example 1: Filter by dependencies
    println!("1. Filter by Dependencies\n");

    let _dep_filter = DependencyFilter::new()
        .package_manager(PackageManager::Npm)
        .has("react")
        .has_at_least("typescript", "4.0")
        .excludes("jquery");

    println!("Filter: NPM projects with React, TypeScript >= 4.0, no jQuery");
    println!(r#"
    let filter = DependencyFilter::new()
        .package_manager(PackageManager::Npm)
        .has("react")
        .has_at_least("typescript", "4.0")
        .excludes("jquery");
"#);

    // Example 2: Filter by framework
    println!("2. Filter by Framework\n");

    let _fw_filter = FrameworkFilter::new()
        .uses(Framework::NextJs)
        .category(FrameworkCategory::Fullstack);

    println!("Filter: Next.js projects (fullstack framework)");
    println!(r#"
    let filter = FrameworkFilter::new()
        .uses(Framework::NextJs)
        .category(FrameworkCategory::Fullstack);
"#);

    // Example 3: Filter by metrics
    println!("3. Filter by Metrics\n");

    let _metric_filter = MetricFilter::new()
        .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
        .file_count(ComparisonOp::LessThan, 500.0)
        .commit_age_days(ComparisonOp::LessThan, 30.0);

    println!("Filter: Projects with >1000 LOC, <500 files, active in last 30 days");
    println!(r#"
    let filter = MetricFilter::new()
        .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
        .file_count(ComparisonOp::LessThan, 500.0)
        .commit_age_days(ComparisonOp::LessThan, 30.0);
"#);

    // Example 4: Filter by language
    println!("4. Filter by Language\n");

    let _lang_filter = LanguageFilter::new()
        .primary(ProgrammingLanguage::TypeScript)
        .min_primary_percentage(50.0)
        .excludes(ProgrammingLanguage::PHP);

    println!("Filter: TypeScript projects (>50% TS), no PHP");
    println!(r#"
    let filter = LanguageFilter::new()
        .primary(ProgrammingLanguage::TypeScript)
        .min_primary_percentage(50.0)
        .excludes(ProgrammingLanguage::PHP);
"#);

    // Example 5: Combined advanced filter
    println!("5. Combined Advanced Filter\n");

    let _advanced = AdvancedRepoFilter::new()
        .with_dependency(
            DependencyFilter::new()
                .has("react")
                .has_at_least("next", "13.0"),
        )
        .with_framework(FrameworkFilter::new().uses(Framework::NextJs))
        .with_metrics(
            MetricFilter::new()
                .lines_of_code(ComparisonOp::GreaterThan, 5000.0),
        )
        .with_language(LanguageFilter::new().primary(ProgrammingLanguage::TypeScript))
        .match_all();

    println!("Combined filter: Next.js 13+ projects, >5000 LOC, TypeScript primary");
    println!(r#"
    let filter = AdvancedRepoFilter::new()
        .with_dependency(
            DependencyFilter::new()
                .has("react")
                .has_at_least("next", "13.0"),
        )
        .with_framework(
            FrameworkFilter::new()
                .uses(Framework::NextJs)
        )
        .with_metrics(
            MetricFilter::new()
                .lines_of_code(ComparisonOp::GreaterThan, 5000.0),
        )
        .with_language(
            LanguageFilter::new()
                .primary(ProgrammingLanguage::TypeScript)
        )
        .match_all();  // All filters must match
"#);

    // Example 6: Using filter presets
    println!("6. Filter Presets\n");

    println!("Available presets:");
    println!("  - FilterPresets::rust_project()");
    println!("  - FilterPresets::typescript_project()");
    println!("  - FilterPresets::react_project()");
    println!("  - FilterPresets::python_project()");
    println!("  - FilterPresets::django_project()");
    println!("  - FilterPresets::go_project()");
    println!("  - FilterPresets::active_project(30.0) // commits in last 30 days");
    println!("  - FilterPresets::large_project(10000.0) // >10000 LOC");

    // Example 7: Integration with Codemod
    println!("\n7. Integration with Codemod Pipeline\n");

    println!(r#"
    // Full example: Upgrade all active React projects in an org
    let result = Codemod::from_github_org("my-org", "ghp_token")
        .repositories(|r| r
            .not_archived()
            .not_fork()
            // Use advanced filtering
            .matches(|path| {{
                AdvancedRepoFilter::new()
                    .with_dependency(
                        DependencyFilter::new()
                            .has("react")
                            .version_below("react", "18.0")
                    )
                    .with_metrics(
                        MetricFilter::new()
                            .commit_age_days(ComparisonOp::LessThan, 90.0)
                    )
                    .matches(path)
                    .unwrap_or(false)
            }}))
        .apply(react_18_upgrade())
        .on_branch("chore/react-18-upgrade")
        .commit_message("chore: upgrade React to v18")
        .push_branch()
        .create_pr("React 18 Upgrade", "Automated upgrade to React 18")
        .execute()?;

    println!("Upgraded {{}} repositories", result.summary.modified_repos);
"#);

    // Example 8: Analyze a repository
    println!("8. Repository Analysis\n");

    println!(r#"
    // Analyze a single repository
    let info = RepositoryInfo::analyze(Path::new("./my-project"))?;

    println!("{{}}", info.summary());

    // Access detailed information
    println!("Primary language: {{:?}}", info.languages.primary);
    println!("Lines of code: {{}}", info.metrics.lines_of_code);
    println!("Frameworks: {{:?}}", info.frameworks.frameworks);
    println!("Package managers: {{:?}}", info.dependencies.package_managers);
"#);

    // Show supported package managers
    println!("\n=== Supported Package Managers ===\n");
    println!("  - Cargo (Rust)");
    println!("  - Npm (JavaScript/TypeScript)");
    println!("  - Yarn (JavaScript/TypeScript)");
    println!("  - Pnpm (JavaScript/TypeScript)");
    println!("  - Pip (Python)");
    println!("  - Poetry (Python)");
    println!("  - Maven (Java)");
    println!("  - Gradle (Java/Kotlin)");
    println!("  - GoMod (Go)");
    println!("  - NuGet (C#/.NET)");
    println!("  - Bundler (Ruby)");
    println!("  - Composer (PHP)");

    // Show supported frameworks
    println!("\n=== Supported Frameworks ===\n");
    println!("JavaScript/TypeScript:");
    println!("  React, NextJs, Vue, NuxtJs, Angular, Svelte, Express, NestJs, Fastify\n");
    println!("Python:");
    println!("  Django, Flask, FastAPI, Pyramid, Tornado\n");
    println!("Ruby:");
    println!("  Rails, Sinatra, Hanami\n");
    println!("Java:");
    println!("  SpringBoot, Quarkus, Micronaut, JakartaEE\n");
    println!("C#:");
    println!("  AspNetCore, Blazor\n");
    println!("Go:");
    println!("  Gin, Echo, Fiber, Chi\n");
    println!("Rust:");
    println!("  ActixWeb, Axum, Rocket, Warp\n");
    println!("PHP:");
    println!("  Laravel, Symfony\n");
    println!("Testing:");
    println!("  Jest, Mocha, Pytest, RSpec, JUnit");

    Ok(())
}
