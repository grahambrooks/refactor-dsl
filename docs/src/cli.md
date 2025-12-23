# CLI Reference

The `refactor` CLI provides command-line access to refactoring operations.

## Installation

```bash
# From source
git clone https://github.com/yourusername/refactor-dsl
cd refactor-dsl
cargo install --path .

# Verify
refactor --version
```

## Commands

### replace

Replace text patterns in files.

```bash
refactor replace [OPTIONS] [PATH]
```

**Arguments:**
- `PATH` - Directory to process (default: current directory)

**Options:**
- `-p, --pattern <REGEX>` - Pattern to search for (required)
- `-r, --replacement <TEXT>` - Replacement text (required)
- `-e, --extension <EXT>` - Filter by file extension
- `-i, --include <GLOB>` - Glob pattern to include
- `--exclude <GLOB>` - Glob pattern to exclude
- `--dry-run` - Preview changes without applying

**Examples:**

```bash
# Replace .unwrap() with .expect() in Rust files
refactor replace \
    --pattern '\.unwrap\(\)' \
    --replacement '.expect("error")' \
    --extension rs \
    --dry-run

# Replace in specific directory, excluding tests
refactor replace \
    --pattern 'old_api' \
    --replacement 'new_api' \
    --extension rs \
    --exclude '**/tests/**' \
    ./src

# Using capture groups
refactor replace \
    --pattern 'fn (\w+)' \
    --replacement 'pub fn $1' \
    --extension rs
```

### find

Find AST patterns in code using tree-sitter queries.

```bash
refactor find [OPTIONS] [PATH]
```

**Arguments:**
- `PATH` - Directory to search (default: current directory)

**Options:**
- `-q, --query <QUERY>` - Tree-sitter query pattern (required)
- `-e, --extension <EXT>` - Filter by file extension

**Examples:**

```bash
# Find all function definitions in Rust
refactor find \
    --query '(function_item name: (identifier) @fn)' \
    --extension rs

# Find struct definitions
refactor find \
    --query '(struct_item name: (type_identifier) @struct)' \
    --extension rs \
    ./src
```

**Output format:**
```
path/to/file.rs:10:5: function_name (fn)
path/to/file.rs:25:1: MyStruct (struct)
```

### rename

Rename symbols across files (text-based, not semantic).

```bash
refactor rename [OPTIONS] [PATH]
```

**Arguments:**
- `PATH` - Directory to process (default: current directory)

**Options:**
- `-f, --from <NAME>` - Original symbol name (required)
- `-t, --to <NAME>` - New symbol name (required)
- `-e, --extension <EXT>` - Filter by file extension
- `--dry-run` - Preview changes without applying

**Examples:**

```bash
# Rename a function
refactor rename \
    --from old_function \
    --to new_function \
    --extension rs \
    --dry-run

# Rename across TypeScript files
refactor rename \
    --from OldComponent \
    --to NewComponent \
    --extension tsx
```

> **Note:** This is a text-based rename. For semantic rename that updates imports and references correctly, use the Rust API with `LspRename`.

### languages

List supported languages for AST operations.

```bash
refactor languages
```

**Output:**
```
Supported languages:
  rust (extensions: rs)
  typescript (extensions: ts, tsx, js, jsx)
  python (extensions: py, pyi)
```

## Global Options

- `--version` - Print version information
- `--help` - Print help information

## Output

### Dry Run Output

When using `--dry-run`, the CLI shows a colorized diff:

```diff
--- src/main.rs
+++ src/main.rs
@@ -10,7 +10,7 @@
 fn process_data() {
-    let result = data.unwrap();
+    let result = data.expect("error");
     process(result);
 }

+1 -1 in 1 file(s)
```

### Apply Output

Without `--dry-run`:

```
Modified 5 file(s)
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Error (invalid arguments, file errors, etc.) |

## Examples

### Common Workflows

**Preview and apply:**
```bash
# Preview
refactor replace -p 'old' -r 'new' -e rs --dry-run

# If satisfied, apply
refactor replace -p 'old' -r 'new' -e rs
```

**Find before replace:**
```bash
# Find occurrences first
refactor find -q '(call_expression function: (identifier) @fn)' -e rs

# Then replace
refactor replace -p 'deprecated_fn' -r 'new_fn' -e rs
```

**Target specific directories:**
```bash
# Only in src/
refactor replace -p 'TODO' -r 'DONE' -i '**/src/**' -e rs

# Exclude generated code
refactor replace -p 'old' -r 'new' --exclude '**/generated/**' -e rs
```

## Shell Completion

Generate shell completions (planned feature):

```bash
# Bash
refactor completions bash > /etc/bash_completion.d/refactor

# Zsh
refactor completions zsh > ~/.zsh/completions/_refactor

# Fish
refactor completions fish > ~/.config/fish/completions/refactor.fish
```

## See Also

- [Getting Started](./getting-started.md)
- [Matchers](./matchers/README.md)
- [Transforms](./transforms/README.md)
- [Tree-sitter Queries](./tree-sitter-queries.md)
