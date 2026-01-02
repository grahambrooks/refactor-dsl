//! GitHub API client using octocrab.

use octocrab::Octocrab;
use std::sync::Arc;
use tokio::runtime::Runtime;

use crate::error::{RefactorError, Result};

/// Client for interacting with the GitHub API.
///
/// This client wraps the octocrab library and provides a synchronous API
/// for compatibility with the rest of the codebase.
#[derive(Clone)]
pub struct GitHubClient {
    pub(crate) token: String,
    pub(crate) base_url: String,
    pub(crate) octocrab: Arc<Octocrab>,
    pub(crate) runtime: Arc<Runtime>,
}

impl GitHubClient {
    /// Create a new GitHub client with the given token.
    pub fn new(token: impl Into<String>) -> Self {
        let token = token.into();
        let octocrab = Octocrab::builder()
            .personal_token(token.clone())
            .build()
            .expect("Failed to build octocrab client");

        let runtime = Runtime::new().expect("Failed to create tokio runtime");

        Self {
            token,
            base_url: "https://api.github.com".into(),
            octocrab: Arc::new(octocrab),
            runtime: Arc::new(runtime),
        }
    }

    /// Create a client for GitHub Enterprise with a custom base URL.
    pub fn with_enterprise(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        let token = token.into();
        let mut url = base_url.into();

        // Remove trailing slash if present
        if url.ends_with('/') {
            url.pop();
        }

        let octocrab = Octocrab::builder()
            .personal_token(token.clone())
            .base_uri(&url)
            .expect("Failed to set base URI")
            .build()
            .expect("Failed to build octocrab client");

        let runtime = Runtime::new().expect("Failed to create tokio runtime");

        Self {
            token,
            base_url: url,
            octocrab: Arc::new(octocrab),
            runtime: Arc::new(runtime),
        }
    }

    /// Create a client using the GITHUB_TOKEN environment variable.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| RefactorError::GitHub {
            message: "GITHUB_TOKEN environment variable not set".into(),
        })?;
        Ok(Self::new(token))
    }

    /// Get the token for use in clone URLs.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Get a reference to the underlying octocrab client.
    pub fn octocrab(&self) -> &Octocrab {
        &self.octocrab
    }

    /// Get a reference to the tokio runtime.
    pub fn runtime(&self) -> &Runtime {
        &self.runtime
    }

    /// Run an async operation synchronously.
    pub(crate) fn block_on<F, T>(&self, future: F) -> T
    where
        F: std::future::Future<Output = T>,
    {
        self.runtime.block_on(future)
    }
}
