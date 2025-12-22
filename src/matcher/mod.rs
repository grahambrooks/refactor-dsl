//! Matcher DSL for filtering repositories, files, and code patterns.

pub mod ast;
pub mod file;
pub mod git;

pub use ast::AstMatcher;
pub use file::FileMatcher;
pub use git::GitMatcher;

use crate::error::Result;
use std::path::{Path, PathBuf};

/// The main matcher builder that combines git, file, and AST matchers.
#[derive(Default, Clone)]
pub struct Matcher {
    git: Option<GitMatcher>,
    file: Option<FileMatcher>,
    ast: Option<AstMatcher>,
}

impl Matcher {
    /// Creates a new empty matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds git repository predicates.
    pub fn git<F>(mut self, f: F) -> Self
    where
        F: FnOnce(GitMatcher) -> GitMatcher,
    {
        self.git = Some(f(GitMatcher::new()));
        self
    }

    /// Adds file matching predicates.
    pub fn files<F>(mut self, f: F) -> Self
    where
        F: FnOnce(FileMatcher) -> FileMatcher,
    {
        self.file = Some(f(FileMatcher::new()));
        self
    }

    /// Adds AST matching predicates.
    pub fn ast<F>(mut self, f: F) -> Self
    where
        F: FnOnce(AstMatcher) -> AstMatcher,
    {
        self.ast = Some(f(AstMatcher::new()));
        self
    }

    /// Tests if a repository matches the git predicates.
    pub fn matches_repo(&self, repo_path: &Path) -> Result<bool> {
        if let Some(ref git) = self.git {
            git.matches(repo_path)
        } else {
            Ok(true)
        }
    }

    /// Collects all files matching the file predicates in the given directory.
    pub fn collect_files(&self, root: &Path) -> Result<Vec<PathBuf>> {
        if let Some(ref file) = self.file {
            file.collect(root)
        } else {
            // Default: collect all files
            FileMatcher::new().collect(root)
        }
    }

    /// Returns the AST matcher if configured.
    pub fn ast_matcher(&self) -> Option<&AstMatcher> {
        self.ast.as_ref()
    }

    /// Returns the file matcher if configured.
    pub fn file_matcher(&self) -> Option<&FileMatcher> {
        self.file.as_ref()
    }

    /// Returns the git matcher if configured.
    pub fn git_matcher(&self) -> Option<&GitMatcher> {
        self.git.as_ref()
    }
}
