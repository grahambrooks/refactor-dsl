//! Git branch operations.

use crate::error::{RefactorError, Result};
use crate::git::GitOps;

/// Branch operations for GitOps.
pub trait BranchOps {
    /// Create a new branch at HEAD.
    fn create_branch(&self, name: &str) -> Result<()>;

    /// Checkout an existing branch.
    fn checkout_branch(&self, name: &str) -> Result<()>;

    /// Create and checkout a new branch (or checkout if it exists).
    fn create_and_checkout(&self, name: &str) -> Result<()>;

    /// Check if a branch exists locally.
    fn branch_exists(&self, name: &str) -> bool;

    /// Get the current branch name.
    fn current_branch(&self) -> Result<String>;

    /// Delete a local branch.
    fn delete_branch(&self, name: &str, force: bool) -> Result<()>;

    /// List all local branches.
    fn list_branches(&self) -> Result<Vec<String>>;
}

impl BranchOps for GitOps {
    fn create_branch(&self, name: &str) -> Result<()> {
        let head = self.repo.head()?;
        let commit = head.peel_to_commit()?;
        self.repo.branch(name, &commit, false)?;
        Ok(())
    }

    fn checkout_branch(&self, name: &str) -> Result<()> {
        let refname = format!("refs/heads/{}", name);

        // Get the reference
        let reference =
            self.repo
                .find_reference(&refname)
                .map_err(|_| RefactorError::BranchError {
                    message: format!("Branch '{}' not found", name),
                })?;

        // Get the tree to checkout
        let obj = reference.peel(git2::ObjectType::Commit)?;

        // Checkout the tree
        self.repo.checkout_tree(&obj, None)?;

        // Set HEAD to the branch
        self.repo.set_head(&refname)?;

        Ok(())
    }

    fn create_and_checkout(&self, name: &str) -> Result<()> {
        if !self.branch_exists(name) {
            self.create_branch(name)?;
        }
        self.checkout_branch(name)
    }

    fn branch_exists(&self, name: &str) -> bool {
        self.repo.find_branch(name, git2::BranchType::Local).is_ok()
    }

    fn current_branch(&self) -> Result<String> {
        let head = self.repo.head()?;

        if head.is_branch() {
            head.shorthand()
                .map(String::from)
                .ok_or_else(|| RefactorError::BranchError {
                    message: "HEAD is detached or has no shorthand name".into(),
                })
        } else {
            Err(RefactorError::BranchError {
                message: "HEAD is not pointing to a branch (detached HEAD state)".into(),
            })
        }
    }

    fn delete_branch(&self, name: &str, force: bool) -> Result<()> {
        let mut branch = self
            .repo
            .find_branch(name, git2::BranchType::Local)
            .map_err(|_| RefactorError::BranchError {
                message: format!("Branch '{}' not found", name),
            })?;

        if force {
            branch.delete()?;
        } else {
            // Check if branch is fully merged before deleting
            let head = self.repo.head()?.peel_to_commit()?;
            let branch_commit = branch.get().peel_to_commit()?;

            if self
                .repo
                .merge_base(head.id(), branch_commit.id())
                .map(|base| base == branch_commit.id())
                .unwrap_or(false)
            {
                branch.delete()?;
            } else {
                return Err(RefactorError::BranchError {
                    message: format!(
                        "Branch '{}' is not fully merged. Use force=true to delete anyway.",
                        name
                    ),
                });
            }
        }

        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<String>> {
        let branches = self.repo.branches(Some(git2::BranchType::Local))?;
        let mut names = Vec::new();

        for branch_result in branches {
            let (branch, _) = branch_result?;
            if let Some(name) = branch.name()? {
                names.push(name.to_string());
            }
        }

        Ok(names)
    }
}
