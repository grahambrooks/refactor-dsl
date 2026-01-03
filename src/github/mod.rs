//! GitHub API integration for multi-repository operations.
//!
//! This module provides a client for interacting with the GitHub API to:
//! - List repositories from organizations or users
//! - Clone repositories
//! - Create pull requests
//!
//! # Example
//!
//! ```rust,no_run
//! use refactor::github::{GitHubClient, RepoOps};
//!
//! let client = GitHubClient::new("ghp_your_token_here");
//!
//! // List all repos in an organization
//! let repos = client.list_org_repos("my-org")?;
//!
//! for repo in repos {
//!     println!("{}: {}", repo.name, repo.clone_url);
//! }
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```

mod client;
mod clone;
mod pr;
mod repos;

pub use client::GitHubClient;
pub use clone::CloneOps;
pub use pr::{CreatePullRequest, PullRequest, PullRequestOps};
pub use repos::{GitHubRepo, RepoOps};
