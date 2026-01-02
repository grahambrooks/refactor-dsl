//! GitHub repository operations.

use crate::error::Result;
use crate::github::GitHubClient;
use serde::Deserialize;

/// Repository information from GitHub API.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubRepo {
    pub id: u64,
    pub name: String,
    pub full_name: String,
    pub clone_url: String,
    pub ssh_url: String,
    pub default_branch: String,
    #[serde(default)]
    pub archived: bool,
    #[serde(default)]
    pub fork: bool,
    #[serde(default)]
    pub topics: Vec<String>,
    pub language: Option<String>,
    pub pushed_at: Option<String>,
    pub description: Option<String>,
    #[serde(rename = "private")]
    pub is_private: bool,
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
        let mut all_repos = Vec::new();
        let mut page = 1;

        loop {
            let endpoint = format!("/orgs/{}/repos?per_page=100&page={}&type=all", org, page);
            let repos: Vec<GitHubRepo> = self.get(&endpoint)?;

            if repos.is_empty() {
                break;
            }

            all_repos.extend(repos);
            page += 1;

            // Safety limit to prevent infinite loops
            if page > 100 {
                break;
            }
        }

        Ok(all_repos)
    }

    fn list_user_repos(&self, user: &str) -> Result<Vec<GitHubRepo>> {
        let mut all_repos = Vec::new();
        let mut page = 1;

        loop {
            let endpoint = format!("/users/{}/repos?per_page=100&page={}", user, page);
            let repos: Vec<GitHubRepo> = self.get(&endpoint)?;

            if repos.is_empty() {
                break;
            }

            all_repos.extend(repos);
            page += 1;

            if page > 100 {
                break;
            }
        }

        Ok(all_repos)
    }

    fn search_repos(&self, query: &str) -> Result<Vec<GitHubRepo>> {
        #[derive(Deserialize)]
        struct SearchResult {
            items: Vec<GitHubRepo>,
        }

        let encoded_query = urlencoding::encode(query);
        let endpoint = format!("/search/repositories?q={}&per_page=100", encoded_query);
        let result: SearchResult = self.get(&endpoint)?;

        Ok(result.items)
    }

    fn get_repo(&self, owner: &str, name: &str) -> Result<GitHubRepo> {
        let endpoint = format!("/repos/{}/{}", owner, name);
        self.get(&endpoint)
    }

    fn list_org_repos_with_topic(&self, org: &str, topic: &str) -> Result<Vec<GitHubRepo>> {
        // Use search API to find repos with specific topic
        let query = format!("org:{} topic:{}", org, topic);
        self.search_repos(&query)
    }
}

/// Extension methods for filtering repository lists.
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
