# Matchers

Matchers define predicates for selecting which repositories, files, and code patterns to include in a refactoring operation.

## Overview

The `Matcher` type combines three kinds of matchers:

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .git(|g| /* Git predicates */)
        .files(|f| /* File predicates */)
        .ast(|a| /* AST predicates */))
    .transform(/* ... */)
    .apply()?;
```

## Matcher Types

### [Git Matcher](./git.md)

Filter repositories by Git state:

```rust
.git(|g| g
    .branch("main")           // Must be on main branch
    .has_file("Cargo.toml")   // Must contain file
    .recent_commits(30)       // Commits within 30 days
    .clean())                 // No uncommitted changes
```

### [File Matcher](./file.md)

Filter files by path, extension, and content:

```rust
.files(|f| f
    .extension("rs")              // Rust files
    .include("**/src/**")         // Only in src/
    .exclude("**/tests/**")       // Exclude tests
    .contains_pattern("TODO"))    // Must contain TODO
```

### [AST Matcher](./ast.md)

Find code patterns using tree-sitter queries:

```rust
.ast(|a| a
    .query("(function_item name: (identifier) @fn)")
    .capture("fn"))
```

## Composition

Matchers compose with AND logic. A file must pass all configured predicates:

```rust
.matching(|m| m
    .git(|g| g.branch("main"))      // Repository must be on main
    .files(|f| f
        .extension("rs")             // File must be .rs
        .exclude("**/target/**")))   // AND not in target/
```

## Default Behavior

If no matcher is specified, all files in the repository are included (except those in `.gitignore`):

```rust
// Matches all files
Refactor::in_repo("./project")
    .transform(/* ... */)
    .apply()?;
```

## Accessing Matchers Directly

You can use matchers independently of the `Refactor` builder:

```rust
use refactor_dsl::prelude::*;

// Check if a repo matches
let git = GitMatcher::new()
    .branch("main")
    .clean();

if git.matches(Path::new("./project"))? {
    println!("Repo is on main and clean");
}

// Collect matching files
let files = FileMatcher::new()
    .extension("rs")
    .exclude("**/target/**")
    .collect(Path::new("./project"))?;

println!("Found {} Rust files", files.len());
```
