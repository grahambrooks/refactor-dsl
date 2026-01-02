//! Repository cloning operations.

use crate::error::{RefactorError, Result};
use crate::github::{GitHubClient, GitHubRepo};
use git2::Repository;
use std::path::{Path, PathBuf};

/// Clone operations for GitHub repositories.
pub trait CloneOps {
    /// Clone a repository to the target directory.
    ///
    /// Returns the path to the cloned repository.
    fn clone_repo(&self, repo: &GitHubRepo, target: &Path) -> Result<PathBuf>;

    /// Clone a repository or update it if it already exists.
    ///
    /// If the repository already exists at the target location, it will be
    /// fetched and reset to match the remote.
    fn clone_or_update(&self, repo: &GitHubRepo, target: &Path) -> Result<PathBuf>;

    /// Clone a repository by owner and name.
    fn clone_by_name(&self, owner: &str, name: &str, target: &Path) -> Result<PathBuf>;
}

impl CloneOps for GitHubClient {
    fn clone_repo(&self, repo: &GitHubRepo, target: &Path) -> Result<PathBuf> {
        let repo_path = target.join(&repo.name);

        if repo_path.exists() {
            return Err(RefactorError::CloneError {
                repo: repo.full_name.clone(),
                message: format!("Directory already exists: {}", repo_path.display()),
            });
        }

        // Use HTTPS URL with token for authentication
        let clone_url = self.authenticated_url(&repo.clone_url);

        Repository::clone(&clone_url, &repo_path).map_err(|e| RefactorError::CloneError {
            repo: repo.full_name.clone(),
            message: format!("Clone failed: {}", e),
        })?;

        Ok(repo_path)
    }

    fn clone_or_update(&self, repo: &GitHubRepo, target: &Path) -> Result<PathBuf> {
        let repo_path = target.join(&repo.name);

        if repo_path.exists() {
            // Update existing repo
            update_existing_repo(&repo_path, &repo.default_branch, repo)?;
            Ok(repo_path)
        } else {
            self.clone_repo(repo, target)
        }
    }

    fn clone_by_name(&self, owner: &str, name: &str, target: &Path) -> Result<PathBuf> {
        use crate::github::RepoOps;
        let repo = self.get_repo(owner, name)?;
        self.clone_repo(&repo, target)
    }
}

impl GitHubClient {
    /// Create an authenticated URL for cloning.
    pub(crate) fn authenticated_url(&self, url: &str) -> String {
        // Convert https://github.com/... to https://token@github.com/...
        if url.starts_with("https://") {
            url.replacen("https://", &format!("https://{}@", self.token), 1)
        } else {
            url.to_string()
        }
    }
}

/// Update an existing repository to match remote.
fn update_existing_repo(repo_path: &Path, default_branch: &str, info: &GitHubRepo) -> Result<()> {
    let repo = Repository::open(repo_path).map_err(|e| RefactorError::CloneError {
        repo: info.full_name.clone(),
        message: format!("Failed to open existing repo: {}", e),
    })?;

    // Fetch from origin
    let mut remote = repo.find_remote("origin").map_err(|e| RefactorError::CloneError {
        repo: info.full_name.clone(),
        message: format!("Failed to find origin remote: {}", e),
    })?;

    remote
        .fetch(&[default_branch], None, None)
        .map_err(|e| RefactorError::CloneError {
            repo: info.full_name.clone(),
            message: format!("Failed to fetch: {}", e),
        })?;

    // Get the fetch head
    let fetch_head = repo
        .find_reference("FETCH_HEAD")
        .map_err(|e| RefactorError::CloneError {
            repo: info.full_name.clone(),
            message: format!("Failed to find FETCH_HEAD: {}", e),
        })?;

    let commit = fetch_head.peel_to_commit().map_err(|e| RefactorError::CloneError {
        repo: info.full_name.clone(),
        message: format!("Failed to get commit: {}", e),
    })?;

    // Reset to the fetched commit
    repo.reset(commit.as_object(), git2::ResetType::Hard, None)
        .map_err(|e| RefactorError::CloneError {
            repo: info.full_name.clone(),
            message: format!("Failed to reset: {}", e),
        })?;

    // Checkout the default branch
    let refname = format!("refs/heads/{}", default_branch);
    if repo.find_reference(&refname).is_ok() {
        repo.set_head(&refname).map_err(|e| RefactorError::CloneError {
            repo: info.full_name.clone(),
            message: format!("Failed to set HEAD: {}", e),
        })?;
    }

    Ok(())
}

/// Batch clone multiple repositories.
#[allow(dead_code)]
pub fn clone_repos(
    client: &GitHubClient,
    repos: &[GitHubRepo],
    target: &Path,
    update_existing: bool,
) -> Vec<(String, Result<PathBuf>)> {
    repos
        .iter()
        .map(|repo| {
            let result = if update_existing {
                client.clone_or_update(repo, target)
            } else {
                client.clone_repo(repo, target)
            };
            (repo.full_name.clone(), result)
        })
        .collect()
}
