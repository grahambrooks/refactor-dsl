//! GitHub API client.

use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, AUTHORIZATION, USER_AGENT};

use crate::error::{RefactorError, Result};

/// Client for interacting with the GitHub API.
#[derive(Clone)]
pub struct GitHubClient {
    pub(crate) token: String,
    pub(crate) base_url: String,
    pub(crate) client: Client,
}

impl GitHubClient {
    /// Create a new GitHub client with the given token.
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            base_url: "https://api.github.com".into(),
            client: Client::new(),
        }
    }

    /// Create a client for GitHub Enterprise with a custom base URL.
    pub fn with_enterprise(token: impl Into<String>, base_url: impl Into<String>) -> Self {
        let mut url = base_url.into();
        // Remove trailing slash if present
        if url.ends_with('/') {
            url.pop();
        }
        Self {
            token: token.into(),
            base_url: url,
            client: Client::new(),
        }
    }

    /// Create a client using the GITHUB_TOKEN environment variable.
    pub fn from_env() -> Result<Self> {
        let token = std::env::var("GITHUB_TOKEN").map_err(|_| RefactorError::GitHub {
            message: "GITHUB_TOKEN environment variable not set".into(),
        })?;
        Ok(Self::new(token))
    }

    /// Get the default headers for API requests.
    pub(crate) fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.token))
                .expect("Invalid token format"),
        );
        headers.insert(
            ACCEPT,
            HeaderValue::from_static("application/vnd.github+json"),
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_static("refactor-dsl"),
        );
        headers.insert(
            "X-GitHub-Api-Version",
            HeaderValue::from_static("2022-11-28"),
        );
        headers
    }

    /// Make a GET request to the GitHub API.
    pub(crate) fn get<T: serde::de::DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .get(&url)
            .headers(self.headers())
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(RefactorError::GitHub {
                message: format!("API request failed ({}): {}", status, body),
            });
        }

        response.json().map_err(|e| RefactorError::GitHub {
            message: format!("Failed to parse response: {}", e),
        })
    }

    /// Make a POST request to the GitHub API.
    pub(crate) fn post<T: serde::de::DeserializeOwned, B: serde::Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        let response = self
            .client
            .post(&url)
            .headers(self.headers())
            .json(body)
            .send()?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().unwrap_or_default();
            return Err(RefactorError::GitHub {
                message: format!("API request failed ({}): {}", status, body),
            });
        }

        response.json().map_err(|e| RefactorError::GitHub {
            message: format!("Failed to parse response: {}", e),
        })
    }

    /// Get the token for use in clone URLs.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Get the base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}
