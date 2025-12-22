//! Git repository matching predicates.

use crate::error::{RefactorError, Result};
use git2::Repository;
use std::path::Path;

/// Predicates for matching Git repositories.
#[derive(Default, Clone)]
pub struct GitMatcher {
    branch: Option<String>,
    has_files: Vec<String>,
    has_remotes: Vec<String>,
    max_commit_age_days: Option<u32>,
    is_clean: Option<bool>,
    has_uncommitted: Option<bool>,
}

impl GitMatcher {
    /// Creates a new git matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Matches repositories on a specific branch.
    pub fn branch(mut self, name: impl Into<String>) -> Self {
        self.branch = Some(name.into());
        self
    }

    /// Matches repositories that contain a specific file.
    pub fn has_file(mut self, path: impl Into<String>) -> Self {
        self.has_files.push(path.into());
        self
    }

    /// Matches repositories with a specific remote.
    pub fn has_remote(mut self, name: impl Into<String>) -> Self {
        self.has_remotes.push(name.into());
        self
    }

    /// Matches repositories with commits in the last N days.
    pub fn recent_commits(mut self, days: u32) -> Self {
        self.max_commit_age_days = Some(days);
        self
    }

    /// Matches repositories with a clean working directory.
    pub fn clean(mut self) -> Self {
        self.is_clean = Some(true);
        self
    }

    /// Matches repositories with uncommitted changes.
    pub fn dirty(mut self) -> Self {
        self.is_clean = Some(false);
        self
    }

    /// Matches repositories with uncommitted changes.
    pub fn has_uncommitted(mut self, has: bool) -> Self {
        self.has_uncommitted = Some(has);
        self
    }

    /// Tests if the repository at the given path matches all predicates.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        let repo = Repository::discover(repo_path)
            .map_err(|_| RefactorError::RepoNotFound(repo_path.to_path_buf()))?;

        // Check branch
        if let Some(ref expected_branch) = self.branch {
            if !self.check_branch(&repo, expected_branch)? {
                return Ok(false);
            }
        }

        // Check required files
        for file in &self.has_files {
            if !repo_path.join(file).exists() {
                return Ok(false);
            }
        }

        // Check remotes
        for remote_name in &self.has_remotes {
            if repo.find_remote(remote_name).is_err() {
                return Ok(false);
            }
        }

        // Check commit recency
        if let Some(max_days) = self.max_commit_age_days {
            if !self.check_recent_commits(&repo, max_days)? {
                return Ok(false);
            }
        }

        // Check clean/dirty state
        if let Some(should_be_clean) = self.is_clean {
            let is_clean = self.check_is_clean(&repo)?;
            if is_clean != should_be_clean {
                return Ok(false);
            }
        }

        // Check uncommitted changes
        if let Some(should_have_uncommitted) = self.has_uncommitted {
            let has = self.check_has_uncommitted(&repo)?;
            if has != should_have_uncommitted {
                return Ok(false);
            }
        }

        Ok(true)
    }

    fn check_branch(&self, repo: &Repository, expected: &str) -> Result<bool> {
        let head = repo.head()?;
        if head.is_branch() {
            if let Some(name) = head.shorthand() {
                return Ok(name == expected);
            }
        }
        Ok(false)
    }

    fn check_recent_commits(&self, repo: &Repository, max_days: u32) -> Result<bool> {
        let head = repo.head()?;
        let commit = head.peel_to_commit()?;
        let commit_time = commit.time();
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs() as i64)
            .unwrap_or(0);

        let age_days = (now - commit_time.seconds()) / (24 * 60 * 60);
        Ok(age_days <= max_days as i64)
    }

    fn check_is_clean(&self, repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(None)?;
        Ok(statuses.is_empty())
    }

    fn check_has_uncommitted(&self, repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(None)?;
        Ok(!statuses.is_empty())
    }
}
