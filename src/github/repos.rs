//! GitHub repository operations using octocrab.

use crate::error::{RefactorError, Result};
use crate::github::GitHubClient;
use octocrab::models::Repository;
use octocrab::params::repos::Sort;

/// Repository information from GitHub API.
///
/// This is our own type that wraps the essential fields from octocrab's Repository model.
#[derive(Debug, Clone)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    pub archived: bool,
    pub fork: bool,
    pub topics: Vec<String>,
    pub language: Option<String>,
    pub pushed_at: Option<String>,
    pub description: Option<String>,
    pub is_private: bool,
}

impl From<Repository> for GitHubRepo {
    fn from(repo: Repository) -> Self {
        Self {
            id: repo.id.0,
            name: repo.name,
            full_name: repo.full_name.unwrap_or_default(),
            clone_url: repo.clone_url.map(|u| u.to_string()).unwrap_or_default(),
            ssh_url: repo.ssh_url.unwrap_or_default(),
            default_branch: repo.default_branch.unwrap_or_else(|| "main".to_string()),
            archived: repo.archived.unwrap_or(false),
            fork: repo.fork.unwrap_or(false),
            topics: repo.topics.unwrap_or_default(),
            language: repo.language.and_then(|v| v.as_str().map(String::from)),
            pushed_at: repo.pushed_at.map(|t| t.to_rfc3339()),
            description: repo.description,
            is_private: repo.private.unwrap_or(false),
        }
    }
}

/// Repository listing and search operations.
pub trait RepoOps {
    /// List all repositories in an organization.
    fn list_org_repos(&self, org: &str) -> Result<Vec<GitHubRepo>>;

    /// List all repositories for a user.
    fn list_user_repos(&self, user: &str) -> Result<Vec<GitHubRepo>>;

    /// Search for repositories.
    fn search_repos(&self, query: &str) -> Result<Vec<GitHubRepo>>;

    /// Get a specific repository.
    fn get_repo(&self, owner: &str, name: &str) -> Result<GitHubRepo>;

    /// List repositories in an organization filtered by topic.
    fn list_org_repos_with_topic(&self, org: &str, topic: &str) -> Result<Vec<GitHubRepo>>;
}

impl RepoOps for GitHubClient {
    fn list_org_repos(&self, org: &str) -> Result<Vec<GitHubRepo>> {
        let octocrab = self.octocrab.clone();
        let org = org.to_string();

        self.block_on(async move {
            let mut all_repos = Vec::new();
            let mut page = 1u32;

            loop {
                let repos = octocrab
                    .orgs(&org)
                    .list_repos()
                    .sort(Sort::Pushed)
                    .per_page(100)
                    .page(page)
                    .send()
                    .await
                    .map_err(|e| RefactorError::GitHub {
                        message: format!("Failed to list org repos: {}", e),
                    })?;

                if repos.items.is_empty() {
                    break;
                }

                all_repos.extend(repos.items.into_iter().map(GitHubRepo::from));
                page += 1;

                // Safety limit
                if page > 100 {
                    break;
                }
            }

            Ok(all_repos)
        })
    }

    fn list_user_repos(&self, user: &str) -> Result<Vec<GitHubRepo>> {
        let octocrab = self.octocrab.clone();
        let user = user.to_string();

        self.block_on(async move {
            let mut all_repos = Vec::new();
            let mut page = 1u32;

            loop {
                let repos = octocrab
                    .users(&user)
                    .repos()
                    .per_page(100)
                    .page(page)
                    .send()
                    .await
                    .map_err(|e| RefactorError::GitHub {
                        message: format!("Failed to list user repos: {}", e),
                    })?;

                if repos.items.is_empty() {
                    break;
                }

                all_repos.extend(repos.items.into_iter().map(GitHubRepo::from));
                page += 1;

                if page > 100 {
                    break;
                }
            }

            Ok(all_repos)
        })
    }

    fn search_repos(&self, query: &str) -> Result<Vec<GitHubRepo>> {
        let octocrab = self.octocrab.clone();
        let query = query.to_string();

        self.block_on(async move {
            let result = octocrab
                .search()
                .repositories(&query)
                .per_page(100)
                .send()
                .await
                .map_err(|e| RefactorError::GitHub {
                    message: format!("Failed to search repos: {}", e),
                })?;

            Ok(result.items.into_iter().map(GitHubRepo::from).collect())
        })
    }

    fn get_repo(&self, owner: &str, name: &str) -> Result<GitHubRepo> {
        let octocrab = self.octocrab.clone();
        let owner = owner.to_string();
        let name = name.to_string();

        self.block_on(async move {
            let repo = octocrab
                .repos(&owner, &name)
                .get()
                .await
                .map_err(|e| RefactorError::GitHub {
                    message: format!("Failed to get repo {}/{}: {}", owner, name, e),
                })?;

            Ok(GitHubRepo::from(repo))
        })
    }

    fn list_org_repos_with_topic(&self, org: &str, topic: &str) -> Result<Vec<GitHubRepo>> {
        // Use search API to find repos with specific topic
        let query = format!("org:{} topic:{}", org, topic);
        self.search_repos(&query)
    }
}

/// Extension methods for filtering repository lists.
#[allow(dead_code)]
pub trait RepoFilterExt {
    /// Filter to non-archived repositories.
    fn active(self) -> Self;

    /// Filter to non-fork repositories.
    fn source_only(self) -> Self;

    /// Filter by language.
    fn with_language(self, lang: &str) -> Self;

    /// Filter by topic.
    fn with_topic(self, topic: &str) -> Self;

    /// Filter by name pattern (case-insensitive contains).
    fn name_contains(self, pattern: &str) -> Self;
}

impl RepoFilterExt for Vec<GitHubRepo> {
    fn active(self) -> Self {
        self.into_iter().filter(|r| !r.archived).collect()
    }

    fn source_only(self) -> Self {
        self.into_iter().filter(|r| !r.fork).collect()
    }

    fn with_language(self, lang: &str) -> Self {
        self.into_iter()
            .filter(|r| {
                r.language
                    .as_ref()
                    .is_some_and(|l| l.eq_ignore_ascii_case(lang))
            })
            .collect()
    }

    fn with_topic(self, topic: &str) -> Self {
        self.into_iter()
            .filter(|r| r.topics.iter().any(|t| t.eq_ignore_ascii_case(topic)))
            .collect()
    }

    fn name_contains(self, pattern: &str) -> Self {
        let pattern_lower = pattern.to_lowercase();
        self.into_iter()
            .filter(|r| r.name.to_lowercase().contains(&pattern_lower))
            .collect()
    }
}
