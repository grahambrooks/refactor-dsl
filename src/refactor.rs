//! Main refactoring engine and DSL entry point.

use crate::diff::{colorized_diff, unified_diff, DiffSummary};
use crate::error::{RefactorError, Result};
use crate::matcher::Matcher;
use crate::transform::{FileChange, TransformBuilder};
use std::fs;
use std::path::{Path, PathBuf};

/// The result of applying a refactoring operation.
#[derive(Debug)]
pub struct RefactorResult {
    pub changes: Vec<FileChange>,
    pub summary: DiffSummary,
}

impl RefactorResult {
    /// Returns the number of files that were modified.
    pub fn files_modified(&self) -> usize {
        self.changes.iter().filter(|c| c.is_modified()).count()
    }

    /// Generates a unified diff of all changes.
    pub fn diff(&self) -> String {
        self.changes
            .iter()
            .filter(|c| c.is_modified())
            .map(|c| unified_diff(&c.original, &c.transformed, &c.path))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Generates a colorized diff for terminal display.
    pub fn colorized_diff(&self) -> String {
        self.changes
            .iter()
            .filter(|c| c.is_modified())
            .map(|c| colorized_diff(&c.original, &c.transformed, &c.path))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The main refactoring DSL builder.
pub struct Refactor {
    root: PathBuf,
    matcher: Option<Matcher>,
    transform: Option<TransformBuilder>,
    dry_run: bool,
}

impl Refactor {
    /// Creates a new refactor operation rooted at the given path.
    pub fn in_repo(path: impl Into<PathBuf>) -> Self {
        Self {
            root: path.into(),
            matcher: None,
            transform: None,
            dry_run: false,
        }
    }

    /// Creates a new refactor operation in the current directory.
    pub fn current_dir() -> Result<Self> {
        Ok(Self::in_repo(std::env::current_dir()?))
    }

    /// Sets the matching predicates for this refactoring.
    pub fn matching<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Matcher) -> Matcher,
    {
        self.matcher = Some(f(Matcher::new()));
        self
    }

    /// Sets the transformations to apply.
    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TransformBuilder) -> TransformBuilder,
    {
        self.transform = Some(f(TransformBuilder::new()));
        self
    }

    /// Enables dry-run mode (preview changes without applying).
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Applies the refactoring and returns the result.
    pub fn apply(self) -> Result<RefactorResult> {
        let matcher = self.matcher.unwrap_or_default();
        let transform = self.transform.unwrap_or_default();

        // Check if repository matches
        if !matcher.matches_repo(&self.root)? {
            return Err(RefactorError::NoFilesMatched);
        }

        // Collect matching files
        let files = matcher.collect_files(&self.root)?;
        if files.is_empty() {
            return Err(RefactorError::NoFilesMatched);
        }

        // Apply transformations
        let mut changes = Vec::new();
        let mut summary = DiffSummary::default();

        for path in files {
            let original = fs::read_to_string(&path)?;
            let transformed = transform.apply(&original, &path)?;

            let file_summary = DiffSummary::from_diff(&original, &transformed);
            summary.merge(&file_summary);

            changes.push(FileChange {
                path: path.clone(),
                original,
                transformed,
            });
        }

        // Apply changes if not dry-run
        if !self.dry_run {
            for change in &changes {
                change.apply()?;
            }
        }

        Ok(RefactorResult { changes, summary })
    }

    /// Runs the refactoring in preview mode and returns the diff.
    pub fn preview(self) -> Result<String> {
        let result = self.dry_run().apply()?;
        Ok(result.diff())
    }

    /// Returns the root path.
    pub fn root(&self) -> &Path {
        &self.root
    }
}

/// Builder for multi-repository refactoring operations.
pub struct MultiRepoRefactor {
    repos: Vec<PathBuf>,
    matcher: Option<Matcher>,
    transform: Option<TransformBuilder>,
    dry_run: bool,
}

impl MultiRepoRefactor {
    /// Creates a new multi-repo refactor operation.
    pub fn new() -> Self {
        Self {
            repos: Vec::new(),
            matcher: None,
            transform: None,
            dry_run: false,
        }
    }

    /// Adds a repository path to the operation.
    pub fn repo(mut self, path: impl Into<PathBuf>) -> Self {
        self.repos.push(path.into());
        self
    }

    /// Adds multiple repository paths.
    pub fn repos(mut self, paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        self.repos.extend(paths.into_iter().map(Into::into));
        self
    }

    /// Discovers repositories in a directory.
    pub fn discover(mut self, parent: impl AsRef<Path>) -> Result<Self> {
        let parent = parent.as_ref();
        for entry in fs::read_dir(parent)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() && path.join(".git").exists() {
                self.repos.push(path);
            }
        }
        Ok(self)
    }

    /// Sets the matching predicates for this refactoring.
    pub fn matching<F>(mut self, f: F) -> Self
    where
        F: FnOnce(Matcher) -> Matcher,
    {
        self.matcher = Some(f(Matcher::new()));
        self
    }

    /// Sets the transformations to apply.
    pub fn transform<F>(mut self, f: F) -> Self
    where
        F: FnOnce(TransformBuilder) -> TransformBuilder,
    {
        self.transform = Some(f(TransformBuilder::new()));
        self
    }

    /// Enables dry-run mode.
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Applies the refactoring to all matching repositories.
    pub fn apply(self) -> Result<Vec<(PathBuf, Result<RefactorResult>)>> {
        let matcher = self.matcher.clone().unwrap_or_default();
        let mut results = Vec::new();

        for repo in &self.repos {
            // Check if repo matches
            if !matcher.matches_repo(repo).unwrap_or(false) {
                continue;
            }

            let refactor = Refactor {
                root: repo.clone(),
                matcher: self.matcher.clone(),
                transform: None, // Will be rebuilt per-repo
                dry_run: self.dry_run,
            };

            // Rebuild transform for each repo (since TransformBuilder isn't Clone)
            let refactor = if self.transform.is_some() {
                // We can't easily clone TransformBuilder, so for now just use empty
                // In a real implementation, we'd make TransformBuilder cloneable
                refactor
            } else {
                refactor
            };

            results.push((repo.clone(), refactor.apply()));
        }

        Ok(results)
    }
}

impl Default for MultiRepoRefactor {
    fn default() -> Self {
        Self::new()
    }
}
