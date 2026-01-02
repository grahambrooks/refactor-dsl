//! Git push operations with authentication.

use crate::error::{RefactorError, Result};
use crate::git::{GitAuth, GitOps};
use git2::{Cred, PushOptions, RemoteCallbacks};

/// Push operations for GitOps.
pub trait PushOps {
    /// Push a branch to a remote.
    fn push(&self, remote_name: &str, branch: &str) -> Result<()>;

    /// Push a branch and set it as upstream.
    fn push_with_upstream(&self, remote_name: &str, branch: &str) -> Result<()>;

    /// Fetch from a remote.
    fn fetch(&self, remote_name: &str, branches: &[&str]) -> Result<()>;

    /// Get the URL for a remote.
    fn remote_url(&self, remote_name: &str) -> Result<String>;

    /// Check if a remote exists.
    fn remote_exists(&self, remote_name: &str) -> bool;

    /// List all remotes.
    fn list_remotes(&self) -> Result<Vec<String>>;
}

impl PushOps for GitOps {
    fn push(&self, remote_name: &str, branch: &str) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name).map_err(|_| {
            RefactorError::PushError {
                message: format!("Remote '{}' not found", remote_name),
            }
        })?;

        let refspec = format!("refs/heads/{}:refs/heads/{}", branch, branch);

        let mut callbacks = RemoteCallbacks::new();
        self.setup_auth_callbacks(&mut callbacks);

        let mut push_options = PushOptions::new();
        push_options.remote_callbacks(callbacks);

        remote
            .push(&[&refspec], Some(&mut push_options))
            .map_err(|e| RefactorError::PushError {
                message: format!("Push failed: {}", e),
            })?;

        Ok(())
    }

    fn push_with_upstream(&self, remote_name: &str, branch: &str) -> Result<()> {
        // First push
        self.push(remote_name, branch)?;

        // Then set upstream tracking
        let mut local_branch = self.repo.find_branch(branch, git2::BranchType::Local)?;
        let upstream_name = format!("{}/{}", remote_name, branch);
        local_branch.set_upstream(Some(&upstream_name))?;

        Ok(())
    }

    fn fetch(&self, remote_name: &str, branches: &[&str]) -> Result<()> {
        let mut remote = self.repo.find_remote(remote_name).map_err(|_| {
            RefactorError::PushError {
                message: format!("Remote '{}' not found", remote_name),
            }
        })?;

        let mut callbacks = RemoteCallbacks::new();
        self.setup_auth_callbacks(&mut callbacks);

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.remote_callbacks(callbacks);

        remote.fetch(branches, Some(&mut fetch_options), None)?;

        Ok(())
    }

    fn remote_url(&self, remote_name: &str) -> Result<String> {
        let remote = self.repo.find_remote(remote_name)?;
        remote
            .url()
            .map(String::from)
            .ok_or_else(|| RefactorError::InvalidConfig(format!("Remote '{}' has no URL", remote_name)))
    }

    fn remote_exists(&self, remote_name: &str) -> bool {
        self.repo.find_remote(remote_name).is_ok()
    }

    fn list_remotes(&self) -> Result<Vec<String>> {
        let remotes = self.repo.remotes()?;
        Ok(remotes.iter().filter_map(|r| r.map(String::from)).collect())
    }
}

impl GitOps {
    fn setup_auth_callbacks(&self, callbacks: &mut RemoteCallbacks<'_>) {
        let auth = self.auth.clone();

        callbacks.credentials(move |_url, username_from_url, allowed_types| {
            match &auth {
                GitAuth::SshKey {
                    private_key_path,
                    passphrase,
                } => {
                    let username = username_from_url.unwrap_or("git");
                    Cred::ssh_key(username, None, private_key_path, passphrase.as_deref())
                }
                GitAuth::Token(token) => {
                    // For HTTPS URLs with token auth
                    Cred::userpass_plaintext(token, "")
                }
                GitAuth::CredentialHelper => {
                    // Try to use the system credential helper
                    if allowed_types.contains(git2::CredentialType::USER_PASS_PLAINTEXT) {
                        Cred::credential_helper(
                            &git2::Config::open_default()?,
                            _url,
                            username_from_url,
                        )
                    } else if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                        // Fall back to SSH agent
                        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
                    } else {
                        Cred::default()
                    }
                }
                GitAuth::None => {
                    // Try SSH agent first, then default
                    if allowed_types.contains(git2::CredentialType::SSH_KEY) {
                        Cred::ssh_key_from_agent(username_from_url.unwrap_or("git"))
                    } else {
                        Cred::default()
                    }
                }
            }
        });

        // Certificate check callback (accept all for now, could be made configurable)
        callbacks.certificate_check(|_cert, _host| Ok(git2::CertificateCheckStatus::CertificateOk));
    }
}
