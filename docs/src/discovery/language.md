# Language Filters

Filter repositories based on the programming languages they use.

## Overview

`LanguageFilter` analyzes repositories to determine:

- **Primary language** - The dominant language by lines of code
- **Language mix** - Percentage breakdown of all languages
- **Language presence** - Whether a specific language is used at all

## Basic Usage

```rust
use refactor::prelude::*;

// Find Rust projects
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .primary_language("rust"))
    .apply(transform)
    .execute()?;
```

## Primary Language

Filter by the dominant language:

```rust
// Primarily Rust
.primary_language("rust")

// Primarily TypeScript
.primary_language("typescript")

// Primarily Python
.primary_language("python")
```

The primary language is determined by lines of code (excluding comments and blanks).

## Language Percentage

Filter by language percentage:

```rust
use refactor::prelude::*;

// At least 80% TypeScript
.language_percentage("typescript", ComparisonOp::GreaterThanOrEqual, 80.0)

// Less than 20% JavaScript (mostly migrated)
.language_percentage("javascript", ComparisonOp::LessThan, 20.0)

// Significant Go presence
.language_percentage("go", ComparisonOp::GreaterThan, 30.0)
```

## Language Presence

Check if any amount of a language is present:

```rust
// Contains any TypeScript
.has_language("typescript")

// Contains any Rust
.has_language("rust")
```

## Multiple Languages

### All Required (AND)

```rust
// TypeScript project with some Rust
.repositories(|r| r
    .primary_language("typescript")
    .has_language("rust"))
```

### Exclude Languages

```rust
// TypeScript without JavaScript
.repositories(|r| r
    .primary_language("typescript")
    .excludes_language("javascript"))
```

### Language Mix

```rust
// Mixed TypeScript/JavaScript projects
.repositories(|r| r
    .language_percentage("typescript", ComparisonOp::GreaterThan, 40.0)
    .language_percentage("javascript", ComparisonOp::GreaterThan, 30.0))
```

## Supported Languages

Languages are detected by file extension:

| Language | Extensions |
|----------|------------|
| Rust | `.rs` |
| TypeScript | `.ts`, `.tsx` |
| JavaScript | `.js`, `.jsx`, `.mjs` |
| Python | `.py`, `.pyi` |
| Go | `.go` |
| Java | `.java` |
| C# | `.cs` |
| Ruby | `.rb` |
| C | `.c`, `.h` |
| C++ | `.cpp`, `.cc`, `.hpp`, `.cxx` |
| Swift | `.swift` |
| Kotlin | `.kt`, `.kts` |
| PHP | `.php` |
| Scala | `.scala` |
| Elixir | `.ex`, `.exs` |
| Haskell | `.hs` |
| Shell | `.sh`, `.bash` |
| HTML | `.html`, `.htm` |
| CSS | `.css` |
| SCSS | `.scss`, `.sass` |
| JSON | `.json` |
| YAML | `.yaml`, `.yml` |
| TOML | `.toml` |
| Markdown | `.md` |

## Direct Usage

```rust
use refactor::discovery::LanguageFilter;

let filter = LanguageFilter::primary("rust");

// Check a single repository
if filter.matches(Path::new("./my-project"))? {
    println!("Project is primarily Rust");
}

// Get language breakdown
let analysis = LanguageFilter::analyze(Path::new("./my-project"))?;

println!("Primary language: {}", analysis.primary);
println!("Language breakdown:");
for (lang, stats) in &analysis.languages {
    println!("  {}: {} lines ({:.1}%)",
        lang, stats.lines, stats.percentage);
}
```

## Language Analysis

Get detailed language statistics:

```rust
use refactor::discovery::LanguageAnalysis;

let analysis = LanguageAnalysis::for_repo(Path::new("./project"))?;

println!("Repository: {}", analysis.repo_path.display());
println!("Primary: {} ({:.1}%)", analysis.primary, analysis.primary_percentage);
println!();
println!("All languages:");
for lang in analysis.ranked() {
    println!("  {}: {} files, {} lines ({:.1}%)",
        lang.name,
        lang.files,
        lang.lines,
        lang.percentage);
}
```

## Configuration

### Exclude Patterns

```rust
let filter = LanguageFilter::primary("rust")
    .exclude_patterns(&[
        "**/target/**",
        "**/vendor/**",
        "**/node_modules/**",
    ]);
```

### Minimum Threshold

Ignore languages below a threshold:

```rust
let filter = LanguageFilter::primary("rust")
    .min_percentage(5.0);  // Ignore languages < 5%
```

### Count by Files vs Lines

```rust
// Default: count by lines
.primary_language("rust")

// Alternative: count by file count
.primary_language_by_files("rust")
```

## Use Cases

### Find Migration Candidates

```rust
// JavaScript projects to migrate to TypeScript
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .primary_language("javascript")
        .excludes_language("typescript"))
    .collect_repos()?;
```

### Find Polyglot Projects

```rust
// Projects using both frontend and backend languages
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .has_language("typescript")
        .has_language("go"))
    .collect_repos()?;
```

### Language Inventory

```rust
// Count language usage across org
let repos = Codemod::from_github_org("company", token)
    .collect_all_repos()?;

let mut lang_counts: HashMap<String, usize> = HashMap::new();
let mut lang_lines: HashMap<String, usize> = HashMap::new();

for repo in &repos {
    let analysis = LanguageAnalysis::for_repo(&repo.path)?;
    for (lang, stats) in &analysis.languages {
        *lang_counts.entry(lang.clone()).or_insert(0) += 1;
        *lang_lines.entry(lang.clone()).or_insert(0) += stats.lines;
    }
}

println!("Language usage across organization:");
for (lang, count) in lang_counts.iter().sorted_by_key(|(_, c)| Reverse(*c)) {
    println!("  {}: {} repos, {} total lines",
        lang, count, lang_lines.get(lang).unwrap_or(&0));
}
```

### Find Pure Language Projects

```rust
// Pure Rust projects (no other languages)
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .language_percentage("rust", ComparisonOp::GreaterThanOrEqual, 95.0))
    .collect_repos()?;
```

## Error Handling

```rust
use refactor::error::RefactorError;

let filter = LanguageFilter::primary("rust");

match filter.matches(Path::new("./project")) {
    Ok(true) => println!("Primarily Rust"),
    Ok(false) => println!("Not primarily Rust"),
    Err(RefactorError::IoError(e)) => {
        println!("Failed to analyze: {}", e);
    }
    Err(e) => return Err(e.into()),
}
```

## Performance

Language analysis is fast but can be optimized:

1. **Exclude vendored code** - Skip `node_modules`, `vendor`, etc.
2. **Cache results** - Analysis is cached by default
3. **Sample large repos** - For very large repos, sample directories

```rust
Codemod::from_github_org("company", token)
    .repositories(|r| r
        .primary_language("rust")
        .cache_duration(Duration::from_secs(3600)))
    .collect_repos()?;
```

## See Also

- [Metrics Filters](./metrics.md) - Filter by code metrics
- [Enhanced Discovery](./README.md) - Full discovery guide
