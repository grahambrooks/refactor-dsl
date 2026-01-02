//! Pull request operations using octocrab.

use crate::error::{RefactorError, Result};
use crate::github::GitHubClient;
use octocrab::models::pulls::PullRequest as OctocrabPR;

/// A pull request on GitHub.
#[derive(Debug, Clone)]
pub struct PullRequest {
    pub id: u64,
    pub number: u64,
    pub html_url: String,
    pub state: String,
    pub title: String,
    pub body: Option<String>,
    pub head: PullRequestRef,
    pub base: PullRequestRef,
    pub draft: bool,
    pub merged: bool,
}

impl From<OctocrabPR> for PullRequest {
    fn from(pr: OctocrabPR) -> Self {
        Self {
            id: pr.id.0,
            number: pr.number,
            html_url: pr.html_url.map(|u| u.to_string()).unwrap_or_default(),
            state: pr.state.map(|s| format!("{:?}", s).to_lowercase()).unwrap_or_default(),
            title: pr.title.unwrap_or_default(),
            body: pr.body,
            head: PullRequestRef {
                ref_name: pr.head.ref_field,
                sha: pr.head.sha,
            },
            base: PullRequestRef {
                ref_name: pr.base.ref_field,
                sha: pr.base.sha,
            },
            draft: pr.draft.unwrap_or(false),
            merged: pr.merged.unwrap_or(false),
        }
    }
}

/// A reference (branch) in a pull request.
#[derive(Debug, Clone)]
pub struct PullRequestRef {
    pub ref_name: String,
    pub sha: String,
}

/// Request body for creating a pull request.
#[derive(Debug, Clone)]
pub struct CreatePullRequest {
    pub title: String,
    pub body: String,
    pub head: String,
    pub base: String,
    pub draft: Option<bool>,
    pub maintainer_can_modify: Option<bool>,
}

impl CreatePullRequest {
    /// Create a new pull request.
    pub fn new(
        title: impl Into<String>,
        body: impl Into<String>,
        head: impl Into<String>,
        base: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            body: body.into(),
            head: head.into(),
            base: base.into(),
            draft: None,
            maintainer_can_modify: None,
        }
    }

    /// Set the pull request as a draft.
    pub fn draft(mut self) -> Self {
        self.draft = Some(true);
        self
    }

    /// Allow maintainers to modify the pull request.
    pub fn maintainer_can_modify(mut self) -> Self {
        self.maintainer_can_modify = Some(true);
        self
    }
}

/// Pull request operations.
pub trait PullRequestOps {
    /// Create a new pull request.
    fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr: CreatePullRequest,
    ) -> Result<PullRequest>;

    /// List open pull requests for a repository.
    fn list_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>>;

    /// Get a specific pull request.
    fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest>;

    /// Check if a pull request exists for a branch.
    fn pull_request_exists(&self, owner: &str, repo: &str, head_branch: &str) -> Result<bool>;
}

impl PullRequestOps for GitHubClient {
    fn create_pull_request(
        &self,
        owner: &str,
        repo: &str,
        pr: CreatePullRequest,
    ) -> Result<PullRequest> {
        let octocrab = self.octocrab.clone();
        let owner = owner.to_string();
        let repo = repo.to_string();

        self.block_on(async move {
            let pulls = octocrab.pulls(&owner, &repo);
            let mut builder = pulls.create(&pr.title, &pr.head, &pr.base);
            builder = builder.body(&pr.body);

            if let Some(true) = pr.draft {
                builder = builder.draft(true);
            }

            if let Some(true) = pr.maintainer_can_modify {
                builder = builder.maintainer_can_modify(true);
            }

            let result = builder.send().await.map_err(|e| {
                let msg = e.to_string();
                if msg.contains("422") || msg.contains("Validation Failed") {
                    RefactorError::PullRequestError {
                        message: format!(
                            "Failed to create PR (branch may not exist or PR already exists): {}",
                            msg
                        ),
                    }
                } else {
                    RefactorError::PullRequestError {
                        message: format!("Failed to create PR: {}", msg),
                    }
                }
            })?;

            Ok(PullRequest::from(result))
        })
    }

    fn list_pull_requests(&self, owner: &str, repo: &str) -> Result<Vec<PullRequest>> {
        let octocrab = self.octocrab.clone();
        let owner = owner.to_string();
        let repo = repo.to_string();

        self.block_on(async move {
            let prs = octocrab
                .pulls(&owner, &repo)
                .list()
                .state(octocrab::params::State::Open)
                .per_page(100)
                .send()
                .await
                .map_err(|e| RefactorError::GitHub {
                    message: format!("Failed to list pull requests: {}", e),
                })?;

            Ok(prs.items.into_iter().map(PullRequest::from).collect())
        })
    }

    fn get_pull_request(&self, owner: &str, repo: &str, number: u64) -> Result<PullRequest> {
        let octocrab = self.octocrab.clone();
        let owner = owner.to_string();
        let repo = repo.to_string();

        self.block_on(async move {
            let pr = octocrab
                .pulls(&owner, &repo)
                .get(number)
                .await
                .map_err(|e| RefactorError::GitHub {
                    message: format!("Failed to get pull request #{}: {}", number, e),
                })?;

            Ok(PullRequest::from(pr))
        })
    }

    fn pull_request_exists(&self, owner: &str, repo: &str, head_branch: &str) -> Result<bool> {
        let prs = self.list_pull_requests(owner, repo)?;
        Ok(prs.iter().any(|pr| pr.head.ref_name == head_branch))
    }
}

/// Builder for creating pull requests with a fluent API.
#[allow(dead_code)]
pub struct PullRequestBuilder<'a> {
    client: &'a GitHubClient,
    owner: String,
    repo: String,
    title: String,
    body: String,
    head: String,
    base: String,
    draft: bool,
}

#[allow(dead_code)]
impl<'a> PullRequestBuilder<'a> {
    /// Create a new pull request builder.
    pub fn new(
        client: &'a GitHubClient,
        owner: impl Into<String>,
        repo: impl Into<String>,
    ) -> Self {
        Self {
            client,
            owner: owner.into(),
            repo: repo.into(),
            title: String::new(),
            body: String::new(),
            head: String::new(),
            base: String::from("main"),
            draft: false,
        }
    }

    /// Set the pull request title.
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    /// Set the pull request body.
    pub fn body(mut self, body: impl Into<String>) -> Self {
        self.body = body.into();
        self
    }

    /// Set the head branch (the branch with changes).
    pub fn head(mut self, head: impl Into<String>) -> Self {
        self.head = head.into();
        self
    }

    /// Set the base branch (the branch to merge into).
    pub fn base(mut self, base: impl Into<String>) -> Self {
        self.base = base.into();
        self
    }

    /// Set the pull request as a draft.
    pub fn draft(mut self) -> Self {
        self.draft = true;
        self
    }

    /// Create the pull request.
    pub fn create(self) -> Result<PullRequest> {
        if self.title.is_empty() {
            return Err(RefactorError::InvalidConfig(
                "Pull request title is required".into(),
            ));
        }
        if self.head.is_empty() {
            return Err(RefactorError::InvalidConfig(
                "Pull request head branch is required".into(),
            ));
        }

        let mut pr = CreatePullRequest::new(&self.title, &self.body, &self.head, &self.base);
        if self.draft {
            pr = pr.draft();
        }

        self.client.create_pull_request(&self.owner, &self.repo, pr)
    }
}
