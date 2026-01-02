//! High-level codemod builder for multi-repository refactoring.
//!
//! This module provides a fluent API for orchestrating code migrations across
//! multiple repositories, with support for:
//!
//! - Fetching repositories from GitHub organizations
//! - Filtering repositories by various criteria
//! - Applying transformations using the existing DSL
//! - Creating branches, committing changes, and pushing
//! - Creating pull requests automatically
//!
//! # Example
//!
//! ```rust,no_run
//! use refactor_dsl::codemod::{Codemod, angular_v4v5_upgrade};
//!
//! let result = Codemod::from_github_org("acme-corp", "ghp_token")
//!     .repositories(|r| r
//!         .has_file("angular.json")
//!         .not_archived()
//!         .not_fork())
//!     .apply(angular_v4v5_upgrade())
//!     .on_branch("chore/angular-upgrade")
//!     .commit_message("chore: upgrade Angular to v15")
//!     .push_branch()
//!     .create_pr("Angular 15 Upgrade", "Automated upgrade...")
//!     .execute()?;
//!
//! println!("Modified {} repositories", result.summary.modified_repos);
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```

mod executor;
mod filter;
mod upgrade;

pub use executor::{CodemodExecutor, CodemodResult, CodemodSummary, RepoResult, RepoStatus};
pub use filter::RepoFilter;
pub use upgrade::{
    angular_v4v5_upgrade, rxjs_5_to_6_upgrade, AngularV4V5Upgrade, ClosureUpgrade, RxJS5To6Upgrade,
    Upgrade,
};

use crate::error::{RefactorError, Result};
use crate::git::GitAuth;
use crate::github::{GitHubClient, GitHubRepo, RepoOps};
use std::path::{Path, PathBuf};

/// Information about a repository being processed.
#[derive(Debug, Clone)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub local_path: PathBuf,
    pub remote_url: Option<String>,
    pub default_branch: String,
    pub topics: Vec<String>,
}

impl From<&GitHubRepo> for RepoInfo {
    fn from(repo: &GitHubRepo) -> Self {
        Self {
            name: repo.name.clone(),
            full_name: repo.full_name.clone(),
            local_path: PathBuf::new(), // Set later during cloning
            remote_url: Some(repo.clone_url.clone()),
            default_branch: repo.default_branch.clone(),
            topics: repo.topics.clone(),
        }
    }
}

/// Source for repositories (trait object for different sources).
pub trait RepositorySource: Send + Sync {
    /// Get the list of repositories, cloning them to the workspace if needed.
    fn get_repositories(&self, workspace: &Path) -> Result<Vec<RepoInfo>>;

    /// Get the GitHub client if this is a GitHub source.
    fn github_client(&self) -> Option<&GitHubClient> {
        None
    }
}

/// GitHub organization as a repository source.
struct GitHubOrgSource {
    client: GitHubClient,
    org: String,
}

impl RepositorySource for GitHubOrgSource {
    fn get_repositories(&self, workspace: &Path) -> Result<Vec<RepoInfo>> {
        use crate::github::CloneOps;

        let repos = self.client.list_org_repos(&self.org)?;
        let mut repo_infos = Vec::new();

        for github_repo in repos {
            let local_path = self.client.clone_or_update(&github_repo, workspace)?;
            let mut info = RepoInfo::from(&github_repo);
            info.local_path = local_path;
            repo_infos.push(info);
        }

        Ok(repo_infos)
    }

    fn github_client(&self) -> Option<&GitHubClient> {
        Some(&self.client)
    }
}

/// Local directory as a repository source.
struct LocalDirSource {
    parent_dir: PathBuf,
}

impl RepositorySource for LocalDirSource {
    fn get_repositories(&self, _workspace: &Path) -> Result<Vec<RepoInfo>> {
        let mut repos = Vec::new();

        for entry in std::fs::read_dir(&self.parent_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() && path.join(".git").exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Try to get the default branch from the repo
                let default_branch = get_default_branch(&path);

                repos.push(RepoInfo {
                    name: name.clone(),
                    full_name: name,
                    local_path: path,
                    remote_url: None,
                    default_branch,
                    topics: Vec::new(),
                });
            }
        }

        Ok(repos)
    }
}

/// Explicit list of paths as a repository source.
struct ExplicitPathSource {
    paths: Vec<PathBuf>,
}

impl RepositorySource for ExplicitPathSource {
    fn get_repositories(&self, _workspace: &Path) -> Result<Vec<RepoInfo>> {
        let mut repos = Vec::new();

        for path in &self.paths {
            if path.join(".git").exists() {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let default_branch = get_default_branch(path);

                repos.push(RepoInfo {
                    name: name.clone(),
                    full_name: name,
                    local_path: path.clone(),
                    remote_url: None,
                    default_branch,
                    topics: Vec::new(),
                });
            }
        }

        Ok(repos)
    }
}

/// The main codemod builder for multi-repository refactoring.
///
/// Provides a fluent API for configuring and executing code migrations
/// across multiple repositories.
pub struct Codemod {
    source: Box<dyn RepositorySource>,
    filter: Option<RepoFilter>,
    upgrades: Vec<Box<dyn Upgrade>>,
    branch_pattern: Option<String>,
    commit_message: Option<String>,
    push_enabled: bool,
    create_pr_enabled: bool,
    pr_title: Option<String>,
    pr_body: Option<String>,
    dry_run: bool,
    workspace: PathBuf,
    git_auth: GitAuth,
}

impl Codemod {
    /// Create a codemod from a GitHub organization.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::Codemod;
    ///
    /// let codemod = Codemod::from_github_org("my-org", "ghp_token");
    /// ```
    pub fn from_github_org(org: impl Into<String>, token: impl Into<String>) -> Self {
        let token = token.into();
        let client = GitHubClient::new(&token);
        Self {
            source: Box::new(GitHubOrgSource {
                client,
                org: org.into(),
            }),
            filter: None,
            upgrades: Vec::new(),
            branch_pattern: None,
            commit_message: None,
            push_enabled: false,
            create_pr_enabled: false,
            pr_title: None,
            pr_body: None,
            dry_run: false,
            workspace: std::env::temp_dir().join("codemod-workspace"),
            git_auth: GitAuth::Token(token),
        }
    }

    /// Create a codemod from a local directory containing multiple repositories.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::Codemod;
    ///
    /// let codemod = Codemod::from_local("/path/to/repos");
    /// ```
    pub fn from_local(parent_dir: impl Into<PathBuf>) -> Self {
        Self {
            source: Box::new(LocalDirSource {
                parent_dir: parent_dir.into(),
            }),
            filter: None,
            upgrades: Vec::new(),
            branch_pattern: None,
            commit_message: None,
            push_enabled: false,
            create_pr_enabled: false,
            pr_title: None,
            pr_body: None,
            dry_run: false,
            workspace: std::env::temp_dir().join("codemod-workspace"),
            git_auth: GitAuth::None,
        }
    }

    /// Create a codemod from explicit repository paths.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::Codemod;
    ///
    /// let codemod = Codemod::from_paths(["/path/to/repo1", "/path/to/repo2"]);
    /// ```
    pub fn from_paths(paths: impl IntoIterator<Item = impl Into<PathBuf>>) -> Self {
        Self {
            source: Box::new(ExplicitPathSource {
                paths: paths.into_iter().map(Into::into).collect(),
            }),
            filter: None,
            upgrades: Vec::new(),
            branch_pattern: None,
            commit_message: None,
            push_enabled: false,
            create_pr_enabled: false,
            pr_title: None,
            pr_body: None,
            dry_run: false,
            workspace: std::env::temp_dir().join("codemod-workspace"),
            git_auth: GitAuth::None,
        }
    }

    /// Filter repositories by predicate.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::Codemod;
    ///
    /// Codemod::from_local("./repos")
    ///     .repositories(|r| r
    ///         .has_file("angular.json")
    ///         .not_archived()
    ///         .not_fork());
    /// ```
    pub fn repositories<F>(mut self, f: F) -> Self
    where
        F: FnOnce(RepoFilter) -> RepoFilter,
    {
        self.filter = Some(f(RepoFilter::new()));
        self
    }

    /// Apply an upgrade transformation.
    ///
    /// Multiple upgrades can be chained and will be applied in order.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::{Codemod, angular_v4v5_upgrade, rxjs_5_to_6_upgrade};
    ///
    /// Codemod::from_local("./repos")
    ///     .apply(angular_v4v5_upgrade())
    ///     .apply(rxjs_5_to_6_upgrade());
    /// ```
    pub fn apply<U: Upgrade + 'static>(mut self, upgrade: U) -> Self {
        self.upgrades.push(Box::new(upgrade));
        self
    }

    /// Create or checkout a branch with the given name.
    ///
    /// Supports placeholders:
    /// - `{repo}` - Repository name
    /// - `{date}` - Current date (YYYY-MM-DD)
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use refactor_dsl::codemod::Codemod;
    ///
    /// Codemod::from_local("./repos")
    ///     .on_branch("chore/upgrade-{date}");
    /// ```
    pub fn on_branch(mut self, pattern: impl Into<String>) -> Self {
        self.branch_pattern = Some(pattern.into());
        self
    }

    /// Set the commit message for changes.
    ///
    /// Supports the same placeholders as `on_branch`.
    pub fn commit_message(mut self, message: impl Into<String>) -> Self {
        self.commit_message = Some(message.into());
        self
    }

    /// Enable pushing the branch to remote after committing.
    pub fn push_branch(mut self) -> Self {
        self.push_enabled = true;
        self
    }

    /// Enable creating a pull request after pushing.
    ///
    /// Requires `push_branch()` to be enabled.
    pub fn create_pr(
        mut self,
        title: impl Into<String>,
        body: impl Into<String>,
    ) -> Self {
        self.create_pr_enabled = true;
        self.pr_title = Some(title.into());
        self.pr_body = Some(body.into());
        self
    }

    /// Enable dry-run mode (preview changes without applying).
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Set the workspace directory for cloning repositories.
    pub fn workspace(mut self, path: impl Into<PathBuf>) -> Self {
        self.workspace = path.into();
        self
    }

    /// Set git authentication for push operations.
    pub fn with_git_auth(mut self, auth: GitAuth) -> Self {
        self.git_auth = auth;
        self
    }

    /// Build and validate the codemod configuration.
    pub fn build(self) -> Result<CodemodExecutor> {
        // Validation
        if self.upgrades.is_empty() {
            return Err(RefactorError::InvalidConfig(
                "No upgrades specified. Use .apply() to add upgrades.".into(),
            ));
        }

        if self.push_enabled && self.branch_pattern.is_none() {
            return Err(RefactorError::InvalidConfig(
                "push_branch() requires on_branch() to be set.".into(),
            ));
        }

        if self.create_pr_enabled && !self.push_enabled {
            return Err(RefactorError::InvalidConfig(
                "create_pr() requires push_branch() to be enabled.".into(),
            ));
        }

        Ok(CodemodExecutor::new(self))
    }

    /// Build and immediately execute the codemod.
    pub fn execute(self) -> Result<CodemodResult> {
        self.build()?.execute()
    }
}

/// Helper function to get the default branch from a git repository.
fn get_default_branch(path: &std::path::Path) -> String {
    git2::Repository::open(path)
        .ok()
        .and_then(|repo| {
            let head = repo.head().ok()?;
            head.shorthand().map(String::from)
        })
        .unwrap_or_else(|| "main".to_string())
}
