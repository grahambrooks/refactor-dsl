# API Reference

This page provides a quick reference to the main types and functions in Refactor DSL.

## Prelude

Import commonly used types with:

```rust
use refactor::prelude::*;
```

This includes:

| Type | Description |
|------|-------------|
| `Refactor` | Main refactoring builder |
| `MultiRepoRefactor` | Multi-repository refactoring |
| `RefactorResult` | Result of a refactoring operation |
| `Matcher` | Combined matcher builder |
| `FileMatcher` | File filtering predicates |
| `GitMatcher` | Git repository predicates |
| `AstMatcher` | AST query matching |
| `Transform` | Transform trait |
| `TransformBuilder` | Composable transform builder |
| `TextTransform` | Text-based transforms |
| `AstTransform` | AST-aware transforms |
| `Language` | Language trait |
| `LanguageRegistry` | Language detection |
| `Rust`, `TypeScript`, `Python` | Built-in languages |
| `LspClient` | LSP protocol client |
| `LspRegistry` | LSP server registry |
| `LspRename` | Semantic rename |
| `LspInstaller` | LSP server installer |
| `LspServerConfig` | Server configuration |
| `RefactorError`, `Result` | Error types |

## Core Types

### Refactor

```rust
impl Refactor {
    // Create refactor for a repository
    fn in_repo(path: impl Into<PathBuf>) -> Self;
    fn current_dir() -> Result<Self>;

    // Configure matching
    fn matching<F>(self, f: F) -> Self
    where F: FnOnce(Matcher) -> Matcher;

    // Configure transforms
    fn transform<F>(self, f: F) -> Self
    where F: FnOnce(TransformBuilder) -> TransformBuilder;

    // Options
    fn dry_run(self) -> Self;

    // Execute
    fn apply(self) -> Result<RefactorResult>;
    fn preview(self) -> Result<String>;

    // Accessors
    fn root(&self) -> &Path;
}
```

### RefactorResult

```rust
impl RefactorResult {
    fn files_modified(&self) -> usize;
    fn diff(&self) -> String;
    fn colorized_diff(&self) -> String;

    // Fields
    changes: Vec<FileChange>,
    summary: DiffSummary,
}
```

### Matcher

```rust
impl Matcher {
    fn new() -> Self;
    fn git<F>(self, f: F) -> Self
    where F: FnOnce(GitMatcher) -> GitMatcher;
    fn files<F>(self, f: F) -> Self
    where F: FnOnce(FileMatcher) -> FileMatcher;
    fn ast<F>(self, f: F) -> Self
    where F: FnOnce(AstMatcher) -> AstMatcher;

    fn matches_repo(&self, path: &Path) -> Result<bool>;
    fn collect_files(&self, root: &Path) -> Result<Vec<PathBuf>>;
}
```

### FileMatcher

```rust
impl FileMatcher {
    fn new() -> Self;
    fn extension(self, ext: impl Into<String>) -> Self;
    fn extensions(self, exts: impl IntoIterator<Item = impl Into<String>>) -> Self;
    fn include(self, pattern: impl Into<String>) -> Self;
    fn exclude(self, pattern: impl Into<String>) -> Self;
    fn contains_pattern(self, pattern: impl Into<String>) -> Self;
    fn name_matches(self, pattern: impl Into<String>) -> Self;
    fn min_size(self, bytes: u64) -> Self;
    fn max_size(self, bytes: u64) -> Self;
    fn collect(&self, root: &Path) -> Result<Vec<PathBuf>>;
}
```

### GitMatcher

```rust
impl GitMatcher {
    fn new() -> Self;
    fn branch(self, name: impl Into<String>) -> Self;
    fn has_file(self, path: impl Into<String>) -> Self;
    fn has_remote(self, name: impl Into<String>) -> Self;
    fn recent_commits(self, days: u32) -> Self;
    fn clean(self) -> Self;
    fn dirty(self) -> Self;
    fn has_uncommitted(self, has: bool) -> Self;
    fn matches(&self, repo_path: &Path) -> Result<bool>;
}
```

### AstMatcher

```rust
impl AstMatcher {
    fn new() -> Self;
    fn query(self, pattern: impl Into<String>) -> Self;
    fn capture(self, name: impl Into<String>) -> Self;
    fn find_matches(&self, source: &str, lang: &dyn Language) -> Result<Vec<AstMatch>>;
    fn find_matches_in_file(&self, path: &Path, registry: &LanguageRegistry) -> Result<Vec<AstMatch>>;
    fn has_matches(&self, source: &str, lang: &dyn Language) -> Result<bool>;
}

struct AstMatch {
    text: String,
    start_byte: usize,
    end_byte: usize,
    start_row: usize,
    start_col: usize,
    end_row: usize,
    end_col: usize,
    capture_name: String,
}
```

### TransformBuilder

```rust
impl TransformBuilder {
    fn new() -> Self;
    fn replace_pattern(self, pattern: &str, replacement: &str) -> Self;
    fn replace_literal(self, needle: &str, replacement: &str) -> Self;
    fn ast<F>(self, f: F) -> Self
    where F: FnOnce(AstTransform) -> AstTransform;
    fn custom<T: Transform + 'static>(self, transform: T) -> Self;
    fn apply(&self, source: &str, path: &Path) -> Result<String>;
    fn describe(&self) -> Vec<String>;
}
```

### TextTransform

```rust
impl TextTransform {
    fn replace(pattern: &str, replacement: &str) -> Self;
    fn replace_regex(pattern: Regex, replacement: impl Into<String>) -> Self;
    fn replace_literal(needle: &str, replacement: &str) -> Self;
    fn prepend_line(pattern: &str, prefix: &str) -> Result<Self>;
    fn append_line(pattern: &str, suffix: &str) -> Result<Self>;
    fn delete_lines(pattern: &str) -> Result<Self>;
    fn insert_after(pattern: &str, content: &str) -> Result<Self>;
    fn insert_before(pattern: &str, content: &str) -> Result<Self>;
}
```

## LSP Types

### LspRename

```rust
impl LspRename {
    fn new(file_path: impl Into<PathBuf>, line: u32, column: u32, new_name: impl Into<String>) -> Self;
    fn find_symbol(file_path: impl Into<PathBuf>, symbol_name: &str, new_name: impl Into<String>) -> Result<Self>;
    fn root(self, path: impl Into<PathBuf>) -> Self;
    fn server(self, config: LspServerConfig) -> Self;
    fn dry_run(self) -> Self;
    fn auto_install(self) -> Self;
    fn execute(self) -> Result<RenameResult>;
}

struct RenameResult {
    workspace_edit: WorkspaceEdit,
    dry_run: bool,
}

impl RenameResult {
    fn file_count(&self) -> usize;
    fn edit_count(&self) -> usize;
    fn is_empty(&self) -> bool;
    fn diff(&self) -> Result<String>;
}
```

### LspInstaller

```rust
impl LspInstaller {
    fn new() -> Result<Self>;
    fn install_dir(self, path: impl Into<PathBuf>) -> Self;
    fn cache_dir(self, path: impl Into<PathBuf>) -> Self;
    fn is_installed(&self, server_name: &str) -> bool;
    fn get_binary_path(&self, server_name: &str) -> Option<PathBuf>;
    fn ensure_installed(&self, server_name: &str) -> Result<PathBuf>;
    fn install(&self, server_name: &str) -> Result<PathBuf>;
    fn list_installed(&self) -> Result<Vec<InstalledServer>>;
    fn uninstall(&self, server_name: &str) -> Result<()>;
}
```

### LspServerConfig

```rust
impl LspServerConfig {
    fn new(name: &str, command: &str) -> Self;
    fn arg(self, arg: impl Into<String>) -> Self;
    fn extensions(self, exts: impl IntoIterator<Item = impl Into<String>>) -> Self;
    fn root_markers(self, markers: impl IntoIterator<Item = impl Into<String>>) -> Self;
    fn find_root(&self, file_path: &Path) -> Option<PathBuf>;
    fn handles_extension(&self, ext: &str) -> bool;
}
```

## Error Types

```rust
pub enum RefactorError {
    Io(std::io::Error),
    Git(git2::Error),
    Glob(globset::Error),
    Regex(regex::Error),
    Query(tree_sitter::QueryError),
    Json(serde_json::Error),
    Parse { path: PathBuf, message: String },
    RepoNotFound(PathBuf),
    UnsupportedLanguage(String),
    NoFilesMatched,
    TransformFailed { message: String },
    InvalidConfig(String),
}

pub type Result<T> = std::result::Result<T, RefactorError>;
```

## See Also

- [Getting Started](./getting-started.md)
- [Examples](./getting-started/quick-start.md)
- Generated rustdoc at [docs.rs](https://docs.rs/refactor)
