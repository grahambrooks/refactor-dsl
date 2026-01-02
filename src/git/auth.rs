//! Git authentication configuration.

use crate::error::{RefactorError, Result};
use std::path::PathBuf;

/// Authentication method for git remote operations.
#[derive(Debug, Clone, Default)]
pub enum GitAuth {
    /// SSH key authentication.
    SshKey {
        private_key_path: PathBuf,
        passphrase: Option<String>,
    },
    /// Token-based authentication (for HTTPS).
    Token(String),
    /// Use system credential helper.
    CredentialHelper,
    /// No authentication (public repos only).
    #[default]
    None,
}

impl GitAuth {
    /// Create SSH key auth from the default location (~/.ssh/id_rsa or ~/.ssh/id_ed25519).
    pub fn ssh_default() -> Result<Self> {
        let home = dirs::home_dir().ok_or_else(|| RefactorError::GitAuth {
            message: "Could not determine home directory".into(),
        })?;

        // Try id_ed25519 first (more modern), then fall back to id_rsa
        let ed25519_path = home.join(".ssh").join("id_ed25519");
        if ed25519_path.exists() {
            return Ok(Self::SshKey {
                private_key_path: ed25519_path,
                passphrase: None,
            });
        }

        let rsa_path = home.join(".ssh").join("id_rsa");
        if rsa_path.exists() {
            return Ok(Self::SshKey {
                private_key_path: rsa_path,
                passphrase: None,
            });
        }

        Err(RefactorError::GitAuth {
            message: "No SSH key found at ~/.ssh/id_ed25519 or ~/.ssh/id_rsa".into(),
        })
    }

    /// Create SSH key auth with a specific key path.
    pub fn ssh_key(path: impl Into<PathBuf>) -> Self {
        Self::SshKey {
            private_key_path: path.into(),
            passphrase: None,
        }
    }

    /// Create SSH key auth with a passphrase.
    pub fn ssh_key_with_passphrase(
        path: impl Into<PathBuf>,
        passphrase: impl Into<String>,
    ) -> Self {
        Self::SshKey {
            private_key_path: path.into(),
            passphrase: Some(passphrase.into()),
        }
    }

    /// Create token-based auth (typically for GitHub/GitLab HTTPS URLs).
    pub fn token(token: impl Into<String>) -> Self {
        Self::Token(token.into())
    }

    /// Create token auth from an environment variable.
    pub fn from_env(var_name: &str) -> Result<Self> {
        let token = std::env::var(var_name).map_err(|_| RefactorError::GitAuth {
            message: format!("Environment variable {} not set", var_name),
        })?;
        Ok(Self::Token(token))
    }

    /// Create token auth from GITHUB_TOKEN environment variable.
    pub fn github_token() -> Result<Self> {
        Self::from_env("GITHUB_TOKEN")
    }

    /// Set passphrase for SSH key auth.
    pub fn with_passphrase(self, passphrase: impl Into<String>) -> Self {
        match self {
            Self::SshKey {
                private_key_path, ..
            } => Self::SshKey {
                private_key_path,
                passphrase: Some(passphrase.into()),
            },
            other => other,
        }
    }
}

