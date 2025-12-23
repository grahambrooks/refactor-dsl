# Quick Start

This guide walks through a complete refactoring workflow.

## Example: Migrating from `unwrap()` to `expect()`

Let's say you want to replace all uses of `.unwrap()` with `.expect("...")` in a Rust project to improve error messages.

### Step 1: Preview Changes

Always start with a dry run to see what will change:

```rust
use refactor_dsl::prelude::*;

fn main() -> Result<()> {
    let result = Refactor::in_repo("./my-rust-project")
        .matching(|m| m
            .git(|g| g.branch("main"))  // Only on main branch
            .files(|f| f
                .extension("rs")
                .exclude("**/target/**")
                .exclude("**/tests/**")))
        .transform(|t| t
            .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO: handle error\")"))
        .dry_run()
        .apply()?;

    // Print colorized diff
    println!("{}", result.colorized_diff());
    println!("\nSummary: {}", result.summary);

    Ok(())
}
```

Output:
```diff
--- src/main.rs
+++ src/main.rs
@@ -10,7 +10,7 @@
 fn process_file(path: &Path) -> String {
-    let content = fs::read_to_string(path).unwrap();
+    let content = fs::read_to_string(path).expect("TODO: handle error");
     content.trim().to_string()
 }

Summary: +1 -1 in 1 file(s)
```

### Step 2: Apply Changes

Once you're satisfied with the preview, remove `.dry_run()`:

```rust
let result = Refactor::in_repo("./my-rust-project")
    .matching(|m| m
        .files(|f| f.extension("rs").exclude("**/target/**")))
    .transform(|t| t
        .replace_pattern(r"\.unwrap\(\)", ".expect(\"TODO: handle error\")"))
    .apply()?;  // Actually apply changes

println!("Modified {} files", result.files_modified());
```

### Step 3: Review with Git

```bash
cd my-rust-project
git diff
git add -p  # Review each change
git commit -m "Replace unwrap() with expect()"
```

## CLI Equivalent

The same operation using the CLI:

```bash
# Preview
refactor replace \
    --pattern '\.unwrap\(\)' \
    --replacement '.expect("TODO: handle error")' \
    --extension rs \
    --exclude '**/target/**' \
    --dry-run \
    ./my-rust-project

# Apply
refactor replace \
    --pattern '\.unwrap\(\)' \
    --replacement '.expect("TODO: handle error")' \
    --extension rs \
    --exclude '**/target/**' \
    ./my-rust-project
```

## Next: More Complex Matching

See [Matchers](../matchers/README.md) to learn how to:
- Filter by Git branch, commits, and repository state
- Use glob patterns and content matching
- Find specific code patterns with AST queries
