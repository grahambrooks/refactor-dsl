//! Git write operations for branch management, commits, and pushing.
//!
//! This module extends the existing git2 integration with write capabilities
//! needed for multi-repository codemod workflows.

mod auth;
mod branch;
mod commit;
mod push;

pub use auth::GitAuth;
pub use branch::BranchOps;
pub use commit::CommitOps;
pub use push::PushOps;

use crate::error::Result;
use git2::Repository;
use std::path::Path;

/// Git operations wrapper with write capabilities.
///
/// Provides a unified interface for git operations needed by the codemod system:
/// - Branch creation and checkout
/// - Staging and committing changes
/// - Pushing to remotes with authentication
///
/// # Example
///
/// ```rust,no_run
/// use refactor::git::{GitOps, GitAuth, BranchOps, CommitOps, PushOps};
///
/// let git = GitOps::open("./my-repo")?
///     .with_auth(GitAuth::ssh_default()?);
///
/// git.create_and_checkout("feature/upgrade")?;
/// // ... make changes ...
/// git.stage_all()?;
/// git.commit("chore: automated upgrade")?;
/// git.push("origin", "feature/upgrade")?;
/// # Ok::<(), refactor::error::RefactorError>(())
/// ```
pub struct GitOps {
    repo: Repository,
    auth: GitAuth,
}

impl GitOps {
    /// Open an existing repository.
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::open(path.as_ref())?;
        Ok(Self {
            repo,
            auth: GitAuth::None,
        })
    }

    /// Discover and open a repository from a path within it.
    pub fn discover(path: impl AsRef<Path>) -> Result<Self> {
        let repo = Repository::discover(path.as_ref())?;
        Ok(Self {
            repo,
            auth: GitAuth::None,
        })
    }

    /// Set authentication method for remote operations.
    pub fn with_auth(mut self, auth: GitAuth) -> Self {
        self.auth = auth;
        self
    }

    /// Get a reference to the underlying git2::Repository.
    pub fn repo(&self) -> &Repository {
        &self.repo
    }

    /// Get the repository's working directory path.
    pub fn workdir(&self) -> Option<&Path> {
        self.repo.workdir()
    }

    /// Get the current authentication configuration.
    pub fn auth(&self) -> &GitAuth {
        &self.auth
    }
}
