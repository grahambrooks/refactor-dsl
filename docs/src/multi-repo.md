# Multi-Repository Refactoring

`MultiRepoRefactor` enables applying the same refactoring operation across multiple repositories at once.

## Basic Usage

```rust
use refactor::prelude::*;

MultiRepoRefactor::new()
    .repo("./project-a")
    .repo("./project-b")
    .repo("./project-c")
    .matching(|m| m
        .git(|g| g.branch("main").clean())
        .files(|f| f.extension("rs")))
    .transform(|t| t
        .replace_literal("old_api", "new_api"))
    .apply()?;
```

## Adding Repositories

### Individual Repos

```rust
MultiRepoRefactor::new()
    .repo("./project-a")
    .repo("./project-b")
```

### Multiple at Once

```rust
MultiRepoRefactor::new()
    .repos(["./project-a", "./project-b", "./project-c"])
```

### Discover in Directory

Find all Git repositories in a parent directory:

```rust
MultiRepoRefactor::new()
    .discover("./workspace")?  // Finds all dirs with .git
```

## Filtering Repositories

Use Git matchers to filter which repositories to process:

```rust
MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m
        .git(|g| g
            .has_file("Cargo.toml")    // Only Rust projects
            .branch("main")             // On main branch
            .recent_commits(30)         // Active in last 30 days
            .clean()))                  // No uncommitted changes
    .transform(/* ... */)
    .apply()?;
```

## Applying Transforms

```rust
MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m
        .git(|g| g.has_file("Cargo.toml"))
        .files(|f| f.extension("rs")))
    .transform(|t| t
        .replace_pattern(r"\.unwrap\(\)", ".expect(\"error\")"))
    .dry_run()  // Preview first!
    .apply()?;
```

## Handling Results

The result is a vector of per-repository results:

```rust
let results = MultiRepoRefactor::new()
    .repos(["./project-a", "./project-b"])
    .transform(|t| t.replace_literal("old", "new"))
    .apply()?;

for (repo_path, result) in results {
    match result {
        Ok(ref_result) => {
            println!("{}: modified {} files",
                repo_path.display(),
                ref_result.files_modified());
        }
        Err(e) => {
            println!("{}: error - {}",
                repo_path.display(),
                e);
        }
    }
}
```

## Dry Run Mode

Always preview changes across all repos first:

```rust
let results = MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m.files(|f| f.extension("rs")))
    .transform(|t| t.replace_literal("old_name", "new_name"))
    .dry_run()
    .apply()?;

for (path, result) in &results {
    if let Ok(r) = result {
        if r.files_modified() > 0 {
            println!("\n=== {} ===", path.display());
            println!("{}", r.diff());
        }
    }
}
```

## Complete Example

Update a deprecated API across all Rust projects in a workspace:

```rust
use refactor::prelude::*;

fn update_api_across_workspace() -> Result<()> {
    let results = MultiRepoRefactor::new()
        .discover("./workspace")?
        .matching(|m| m
            // Only Rust projects on main branch
            .git(|g| g
                .has_file("Cargo.toml")
                .branch("main")
                .clean())
            // Only .rs files with the old API
            .files(|f| f
                .extension("rs")
                .exclude("**/target/**")
                .contains_pattern("deprecated_function")))
        .transform(|t| t
            .replace_pattern(
                r"deprecated_function\((.*?)\)",
                "new_function($1, Default::default())"
            ))
        .dry_run()
        .apply()?;

    // Summary
    let mut total_files = 0;
    let mut repos_modified = 0;

    for (path, result) in &results {
        if let Ok(r) = result {
            let files = r.files_modified();
            if files > 0 {
                repos_modified += 1;
                total_files += files;
                println!("{}: {} files", path.display(), files);
            }
        }
    }

    println!("\nTotal: {} files across {} repositories",
        total_files, repos_modified);

    Ok(())
}
```

## Error Handling

Individual repository failures don't stop the whole operation:

```rust
let results = multi_refactor.apply()?;

let (successes, failures): (Vec<_>, Vec<_>) = results
    .into_iter()
    .partition(|(_, r)| r.is_ok());

println!("Succeeded: {} repos", successes.len());
println!("Failed: {} repos", failures.len());

for (path, err) in failures {
    println!("  {}: {}", path.display(), err.unwrap_err());
}
```

## Use Cases

### Dependency Updates

```rust
// Update version in all Cargo.toml files
MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m
        .files(|f| f.name_matches(r"^Cargo\.toml$")))
    .transform(|t| t
        .replace_pattern(
            r#"my-lib = "1\.0""#,
            r#"my-lib = "2.0""#
        ))
    .apply()?;
```

### Code Style Enforcement

```rust
// Add missing newlines at end of files
MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m
        .git(|g| g.has_file("Cargo.toml"))
        .files(|f| f.extension("rs")))
    .transform(|t| t
        .replace_pattern(r"([^\n])$", "$1\n"))
    .apply()?;
```

### License Header Updates

```rust
MultiRepoRefactor::new()
    .discover("./workspace")?
    .matching(|m| m
        .files(|f| f
            .extension("rs")
            .contains_pattern("// Copyright 2023")))
    .transform(|t| t
        .replace_literal("// Copyright 2023", "// Copyright 2024"))
    .apply()?;
```

## Limitations

- Transforms are applied independently to each repo
- No cross-repository dependency resolution
- Large workspaces may be slow (consider parallel processing)

## Tips

1. **Always use dry_run first** - Review changes before applying
2. **Use specific matchers** - Avoid modifying unexpected repos
3. **Require clean state** - Use `.git(|g| g.clean())` to avoid conflicts
4. **Check branch** - Ensure you're on the right branch
5. **Commit separately** - Each repo should be committed individually
