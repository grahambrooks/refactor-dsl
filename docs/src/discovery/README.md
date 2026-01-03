# Enhanced Repository Discovery

Refactor DSL provides advanced repository discovery and filtering capabilities, enabling you to target specific repositories in large organizations based on dependencies, frameworks, metrics, and languages.

## Overview

The discovery module extends basic Git repository filtering with:

- **Dependency filtering** - Find repos using specific packages
- **Framework detection** - Identify repos using React, Rails, Spring, etc.
- **Metrics filtering** - Filter by lines of code, file count, complexity
- **Language detection** - Target repos by primary programming language

## Quick Start

```rust
use refactor_dsl::prelude::*;

// Find all React 17+ projects with over 1000 lines of code
Codemod::from_github_org("acme-corp", token)
    .repositories(|r| r
        .has_dependency("react", ">=17.0")
        .uses_framework(Framework::React)
        .lines_of_code(ComparisonOp::GreaterThan, 1000.0))
    .apply(upgrade_operation)
    .execute()?;
```

## Filter Types

### [Dependency Filters](./dependency.md)

Filter by package dependencies:

```rust
.repositories(|r| r
    // NPM packages
    .has_dependency("react", ">=17.0")
    .has_dependency("typescript", "*")

    // Cargo crates
    .has_dependency("tokio", ">=1.0")

    // Python packages
    .has_dependency("django", ">=3.0")
)
```

### [Framework Filters](./framework.md)

Filter by detected frameworks:

```rust
.repositories(|r| r
    .uses_framework(Framework::NextJs)
    // or
    .uses_framework(Framework::Rails)
    // or
    .uses_framework(Framework::Spring)
)
```

### [Metrics Filters](./metrics.md)

Filter by code metrics:

```rust
.repositories(|r| r
    .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
    .file_count(ComparisonOp::LessThan, 100.0)
    .complexity(ComparisonOp::LessThan, 20.0)
)
```

### [Language Filters](./language.md)

Filter by primary language:

```rust
.repositories(|r| r
    .primary_language("rust")
    // Or by percentage
    .language_percentage("typescript", ComparisonOp::GreaterThan, 50.0)
)
```

## Combining Filters

Filters combine with AND logic:

```rust
Codemod::from_github_org("company", token)
    .repositories(|r| r
        // All conditions must be true
        .primary_language("typescript")
        .uses_framework(Framework::React)
        .has_dependency("react", ">=18.0")
        .lines_of_code(ComparisonOp::GreaterThan, 5000.0)
    )
    .apply(migration)
    .execute()?;
```

For OR logic, use multiple discovery passes:

```rust
// Find repos using either React OR Vue
let react_repos = discover_repos()
    .uses_framework(Framework::React)
    .collect()?;

let vue_repos = discover_repos()
    .uses_framework(Framework::Vue)
    .collect()?;

let all_frontend_repos: HashSet<_> = react_repos.union(&vue_repos).collect();
```

## Discovery Sources

### GitHub Organization

```rust
Codemod::from_github_org("organization-name", github_token)
    .repositories(|r| r.has_file("Cargo.toml"))
    .apply(transform)
    .execute()?;
```

### GitHub User

```rust
Codemod::from_github_user("username", github_token)
    .repositories(|r| r.primary_language("rust"))
    .apply(transform)
    .execute()?;
```

### Local Directory

```rust
Codemod::from_directory("./workspace")
    .repositories(|r| r.uses_framework(Framework::Django))
    .apply(transform)
    .execute()?;
```

### Custom Sources

```rust
let repos = vec![
    PathBuf::from("./project-a"),
    PathBuf::from("./project-b"),
    PathBuf::from("./project-c"),
];

Codemod::from_paths(repos)
    .repositories(|r| r.has_dependency("lodash", "*"))
    .apply(transform)
    .execute()?;
```

## Advanced Repository Filter

Use `AdvancedRepoFilter` for complex filtering:

```rust
use refactor_dsl::discovery::AdvancedRepoFilter;

let filter = AdvancedRepoFilter::new()
    // Base Git filters
    .branch("main")
    .has_file("package.json")
    .recent_commits(30)

    // Dependency filters
    .dependency(DependencyFilter::npm("react", ">=17"))
    .dependency(DependencyFilter::npm("typescript", "*"))

    // Framework detection
    .framework(FrameworkFilter::new(Framework::NextJs))

    // Metrics
    .metric(MetricFilter::lines_of_code(ComparisonOp::GreaterThan, 1000.0))

    // Language
    .language(LanguageFilter::primary("typescript"));

// Apply filter
let matching_repos = filter.discover("./workspace")?;
```

## Caching and Performance

Discovery operations cache results for performance:

```rust
Codemod::from_github_org("large-org", token)
    .repositories(|r| r
        .cache_duration(Duration::from_secs(3600))  // Cache for 1 hour
        .parallel_discovery(true)  // Parallel analysis
        .has_dependency("react", "*"))
    .apply(transform)
    .execute()?;
```

## Error Handling

```rust
use refactor_dsl::error::RefactorError;

match Codemod::from_github_org("org", token)
    .repositories(|r| r.has_dependency("react", "*"))
    .apply(transform)
    .execute()
{
    Ok(results) => {
        println!("Processed {} repositories", results.len());
    }
    Err(RefactorError::GithubApiError(msg)) => {
        println!("GitHub API error: {}", msg);
    }
    Err(RefactorError::RateLimited(retry_after)) => {
        println!("Rate limited. Retry after {} seconds", retry_after);
    }
    Err(e) => return Err(e.into()),
}
```

## Use Cases

### Organization-Wide Dependency Updates

```rust
// Update lodash to v4.17.21 across all JS projects
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .has_dependency("lodash", "<4.17.21"))
    .apply(|ctx| {
        ctx.update_dependency("lodash", "4.17.21")
    })
    .create_prs(true)
    .execute()?;
```

### Framework Migration

```rust
// Find all Create React App projects for Next.js migration
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .has_dependency("react-scripts", "*")
        .lines_of_code(ComparisonOp::LessThan, 10000.0))  // Small projects first
    .collect_repos()?;
```

### Security Auditing

```rust
// Find repos with vulnerable dependency versions
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .has_dependency("log4j", "<2.17.0"))  // Vulnerable version
    .apply(|ctx| {
        ctx.create_security_issue()
    })
    .execute()?;
```

### Language Standardization

```rust
// Find all TypeScript projects not using strict mode
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .primary_language("typescript")
        .has_file("tsconfig.json"))
    .apply(|ctx| {
        ctx.ensure_strict_mode()
    })
    .execute()?;
```

## See Also

- [Dependency Filters](./dependency.md) - Package dependency filtering
- [Framework Filters](./framework.md) - Framework detection
- [Metrics Filters](./metrics.md) - Code metrics filtering
- [Language Filters](./language.md) - Language-based filtering
- [Multi-Repository](../multi-repo.md) - Basic multi-repo operations
