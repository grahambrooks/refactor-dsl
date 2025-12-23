# Git Matcher

The `GitMatcher` filters repositories based on Git state, including branch, commits, remotes, and working tree status.

## Basic Usage

```rust
use refactor_dsl::prelude::*;

let matcher = GitMatcher::new()
    .branch("main")
    .clean();

if matcher.matches(Path::new("./project"))? {
    println!("Repository is on main branch with clean working tree");
}
```

## Methods

### Branch Matching

```rust
// Must be on specific branch
.branch("main")
.branch("develop")
.branch("feature/my-feature")
```

### File Existence

Check if specific files exist in the repository:

```rust
// Rust project
.has_file("Cargo.toml")

// Node.js project
.has_file("package.json")

// Has both (AND logic)
.has_file("Cargo.toml")
.has_file("rust-toolchain.toml")
```

### Remote Matching

Check for specific Git remotes:

```rust
// Has origin remote
.has_remote("origin")

// Has upstream remote
.has_remote("upstream")
```

### Commit Recency

Match repositories with recent activity:

```rust
// Has commits within last 30 days
.recent_commits(30)

// Active in last week
.recent_commits(7)

// Very recent activity
.recent_commits(1)
```

### Working Tree State

```rust
// Clean working tree (no uncommitted changes)
.clean()

// Has uncommitted changes
.dirty()

// Explicit check
.has_uncommitted(true)   // Same as .dirty()
.has_uncommitted(false)  // Same as .clean()
```

## Complete Example

```rust
use refactor_dsl::prelude::*;

fn find_active_rust_projects(workspace: &Path) -> Result<Vec<PathBuf>> {
    let matcher = GitMatcher::new()
        .branch("main")
        .has_file("Cargo.toml")
        .recent_commits(30)
        .clean();

    let mut matching_repos = Vec::new();

    for entry in fs::read_dir(workspace)? {
        let path = entry?.path();
        if path.join(".git").exists() && matcher.matches(&path)? {
            matching_repos.push(path);
        }
    }

    Ok(matching_repos)
}
```

## Integration with Refactor

### Single Repository

```rust
Refactor::in_repo("./project")
    .matching(|m| m
        .git(|g| g
            .branch("main")
            .clean())
        .files(|f| f.extension("rs")))
    .transform(/* ... */)
    .apply()?;
```

### Multi-Repository

```rust
MultiRepoRefactor::new()
    .discover("./workspace")?  // Find all git repos in workspace
    .matching(|m| m
        .git(|g| g
            .has_file("Cargo.toml")
            .recent_commits(30)
            .clean()))
    .transform(|t| t.replace_literal("old_name", "new_name"))
    .apply()?;
```

## Error Handling

Git matcher operations can fail if:
- Path is not a Git repository
- Repository is corrupted
- Git operations fail

```rust
use refactor_dsl::error::RefactorError;

match matcher.matches(path) {
    Ok(true) => println!("Matches"),
    Ok(false) => println!("Does not match"),
    Err(RefactorError::RepoNotFound(p)) => {
        println!("Not a git repo: {}", p.display());
    }
    Err(e) => return Err(e.into()),
}
```

## Use Cases

### Only Modify Production-Ready Code

```rust
.git(|g| g
    .branch("main")
    .clean())
```

### Find Stale Repositories

```rust
let stale = GitMatcher::new()
    .has_file("Cargo.toml");
    // Note: No recent_commits filter - check manually

// Then filter by age > 90 days in your code
```

### Exclude Work-in-Progress

```rust
.git(|g| g
    .clean()  // No uncommitted changes
    .has_uncommitted(false))
```
