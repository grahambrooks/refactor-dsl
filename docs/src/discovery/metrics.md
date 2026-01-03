# Metrics Filters

Filter repositories based on code metrics like lines of code, file count, complexity, and more.

## Overview

`MetricFilter` analyzes repositories to compute metrics and filter based on thresholds:

- **Lines of code** - Total source lines
- **File count** - Number of source files
- **Average file size** - Mean lines per file
- **Cyclomatic complexity** - Code complexity measure
- **Test coverage** - If coverage data is available
- **Commit activity** - Recent commit patterns

## Basic Usage

```rust
use refactor_dsl::prelude::*;

// Find large projects
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .lines_of_code(ComparisonOp::GreaterThan, 10000.0))
    .apply(transform)
    .execute()?;
```

## Comparison Operators

```rust
pub enum ComparisonOp {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    Equal,
    NotEqual,
}
```

## Available Metrics

### Lines of Code

```rust
// Large projects
.lines_of_code(ComparisonOp::GreaterThan, 50000.0)

// Small projects
.lines_of_code(ComparisonOp::LessThan, 1000.0)

// Medium projects
.lines_of_code(ComparisonOp::GreaterThanOrEqual, 5000.0)
.lines_of_code(ComparisonOp::LessThan, 20000.0)
```

### File Count

```rust
// Many files
.file_count(ComparisonOp::GreaterThan, 100.0)

// Few files (microservices)
.file_count(ComparisonOp::LessThan, 20.0)
```

### Average File Size

```rust
// Large files (might need splitting)
.avg_file_size(ComparisonOp::GreaterThan, 500.0)

// Small, focused files
.avg_file_size(ComparisonOp::LessThan, 200.0)
```

### Cyclomatic Complexity

```rust
// High complexity (might need refactoring)
.complexity(ComparisonOp::GreaterThan, 15.0)

// Low complexity
.complexity(ComparisonOp::LessThan, 10.0)
```

### Test Coverage

```rust
// Well-tested projects
.coverage(ComparisonOp::GreaterThanOrEqual, 80.0)

// Under-tested projects
.coverage(ComparisonOp::LessThan, 50.0)
```

### Commit Activity

```rust
// Active projects (commits in last N days)
.commits_in_days(30, ComparisonOp::GreaterThan, 5.0)

// Stale projects
.commits_in_days(90, ComparisonOp::LessThan, 1.0)
```

## Language-Specific Metrics

Count only specific languages:

```rust
// TypeScript lines only
.lines_of_code_for("typescript", ComparisonOp::GreaterThan, 5000.0)

// Rust files only
.file_count_for("rust", ComparisonOp::GreaterThan, 10.0)
```

## Multiple Metrics

Combine metrics (AND logic):

```rust
.repositories(|r| r
    // Medium-sized, well-structured projects
    .lines_of_code(ComparisonOp::GreaterThan, 5000.0)
    .lines_of_code(ComparisonOp::LessThan, 50000.0)
    .avg_file_size(ComparisonOp::LessThan, 300.0)
    .complexity(ComparisonOp::LessThan, 12.0)
)
```

## Direct Usage

```rust
use refactor_dsl::discovery::MetricFilter;

let filter = MetricFilter::lines_of_code(ComparisonOp::GreaterThan, 1000.0);

// Check a single repository
if filter.matches(Path::new("./my-project"))? {
    println!("Project has more than 1000 lines");
}

// Get the actual metric value
let metrics = MetricFilter::compute_metrics(Path::new("./my-project"))?;
println!("Lines of code: {}", metrics.lines_of_code);
println!("File count: {}", metrics.file_count);
println!("Average file size: {:.1}", metrics.avg_file_size);
```

## Metrics Report

Generate a full metrics report:

```rust
use refactor_dsl::discovery::MetricsReport;

let report = MetricsReport::for_repo(Path::new("./project"))?;

println!("Metrics Report for {}", report.path.display());
println!("================");
println!("Lines of code: {}", report.lines_of_code);
println!("Files: {}", report.file_count);
println!("Average file size: {:.1} lines", report.avg_file_size);
println!("Languages:");
for (lang, count) in &report.lines_by_language {
    println!("  {}: {} lines ({:.1}%)",
        lang, count, (*count as f64 / report.lines_of_code as f64) * 100.0);
}

// Export as JSON
let json = report.to_json()?;
```

## Exclusions

Configure what to exclude from metrics:

```rust
let filter = MetricFilter::lines_of_code(ComparisonOp::GreaterThan, 1000.0)
    .exclude_patterns(&[
        "**/node_modules/**",
        "**/target/**",
        "**/vendor/**",
        "**/*.min.js",
        "**/generated/**",
    ])
    .exclude_languages(&["json", "yaml", "xml"]);
```

## Weighted Metrics

Create composite metrics:

```rust
use refactor_dsl::discovery::CompositeMetric;

// "Maintainability score"
let maintainability = CompositeMetric::new()
    .add(MetricType::LinesOfCode, 0.3, |loc| {
        // Score decreases with size
        1.0 - (loc / 100000.0).min(1.0)
    })
    .add(MetricType::Complexity, 0.4, |c| {
        // Score decreases with complexity
        1.0 - (c / 25.0).min(1.0)
    })
    .add(MetricType::Coverage, 0.3, |cov| {
        // Score increases with coverage
        cov / 100.0
    });

.repositories(|r| r
    .composite_metric(maintainability, ComparisonOp::GreaterThan, 0.7))
```

## Use Cases

### Find Large Projects for Migration

```rust
// Large TypeScript projects that might benefit from strict mode
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .primary_language("typescript")
        .lines_of_code(ComparisonOp::GreaterThan, 10000.0))
    .apply(enable_strict_mode)
    .execute()?;
```

### Find Stale Projects

```rust
// Projects with no recent activity
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .commits_in_days(180, ComparisonOp::LessThan, 1.0))
    .collect_repos()?;
```

### Find Complex Code

```rust
// High-complexity projects needing refactoring
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .complexity(ComparisonOp::GreaterThan, 20.0)
        .lines_of_code(ComparisonOp::GreaterThan, 5000.0))
    .collect_repos()?;
```

### Prioritize by Size

```rust
// Start with small projects for gradual rollout
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .uses_framework(Framework::React)
        .lines_of_code(ComparisonOp::LessThan, 5000.0))
    .apply(upgrade)
    .execute()?;
```

## Error Handling

```rust
use refactor_dsl::error::RefactorError;

let filter = MetricFilter::lines_of_code(ComparisonOp::GreaterThan, 1000.0);

match filter.matches(Path::new("./project")) {
    Ok(true) => println!("Matches"),
    Ok(false) => println!("Doesn't match"),
    Err(RefactorError::IoError(e)) => {
        println!("Failed to read files: {}", e);
    }
    Err(e) => return Err(e.into()),
}
```

## Performance Considerations

Metrics computation can be slow for large repositories:

1. **Use caching** - Results are cached by default
2. **Limit depth** - Exclude `node_modules`, `target`, etc.
3. **Sample for estimates** - For very large repos, sample a subset
4. **Run in parallel** - Discovery parallelizes across repos

```rust
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
        .cache_duration(Duration::from_secs(3600))
        .parallel_discovery(true))
    .collect_repos()?;
```

## See Also

- [Language Filters](./language.md) - Filter by programming language
- [Enhanced Discovery](./README.md) - Full discovery guide
