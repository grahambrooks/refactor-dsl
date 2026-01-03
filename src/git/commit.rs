//! Git commit operations.

use crate::error::{RefactorError, Result};
use crate::git::GitOps;
use git2::{IndexAddOption, Signature};
use std::path::Path;

/// Commit operations for GitOps.
pub trait CommitOps {
    /// Stage all changes (new, modified, deleted files).
    fn stage_all(&self) -> Result<()>;

    /// Stage specific files.
    fn stage_files(&self, paths: &[&Path]) -> Result<()>;

    /// Stage files matching a pattern.
    fn stage_pattern(&self, pattern: &str) -> Result<()>;

    /// Unstage all files.
    fn unstage_all(&self) -> Result<()>;

    /// Create a commit with the staged changes.
    fn commit(&self, message: &str) -> Result<git2::Oid>;

    /// Create a commit with custom author/committer.
    fn commit_with_signature(
        &self,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<git2::Oid>;

    /// Check if there are staged changes.
    fn has_staged_changes(&self) -> Result<bool>;

    /// Check if there are unstaged changes.
    fn has_unstaged_changes(&self) -> Result<bool>;

    /// Check if the working directory is clean (no changes at all).
    fn is_clean(&self) -> Result<bool>;
}

impl CommitOps for GitOps {
    fn stage_all(&self) -> Result<()> {
        let mut index = self.repo.index()?;

        // Add all files (including new, modified, and handle deleted)
        index.add_all(["*"].iter(), IndexAddOption::DEFAULT, None)?;

        // Update the index for deleted files
        index.update_all(["*"].iter(), None)?;

        index.write()?;
        Ok(())
    }

    fn stage_files(&self, paths: &[&Path]) -> Result<()> {
        let mut index = self.repo.index()?;

        for path in paths {
            // Get path relative to repo root
            let workdir = self.repo.workdir().ok_or_else(|| {
                RefactorError::Git(git2::Error::from_str("Repository has no working directory"))
            })?;

            let rel_path = path.strip_prefix(workdir).unwrap_or(path);

            if path.exists() {
                index.add_path(rel_path)?;
            } else {
                // File was deleted
                index.remove_path(rel_path)?;
            }
        }

        index.write()?;
        Ok(())
    }

    fn stage_pattern(&self, pattern: &str) -> Result<()> {
        let mut index = self.repo.index()?;
        index.add_all([pattern].iter(), IndexAddOption::DEFAULT, None)?;
        index.write()?;
        Ok(())
    }

    fn unstage_all(&self) -> Result<()> {
        let head = self.repo.head()?.peel_to_commit()?;
        self.repo
            .reset(head.as_object(), git2::ResetType::Mixed, None)?;
        Ok(())
    }

    fn commit(&self, message: &str) -> Result<git2::Oid> {
        let signature = self.get_signature()?;
        self.commit_internal(message, &signature, &signature)
    }

    fn commit_with_signature(
        &self,
        message: &str,
        author_name: &str,
        author_email: &str,
    ) -> Result<git2::Oid> {
        let author = Signature::now(author_name, author_email)?;
        let committer = self.get_signature()?;
        self.commit_internal(message, &author, &committer)
    }

    fn has_staged_changes(&self) -> Result<bool> {
        let head = self.repo.head()?.peel_to_tree()?;
        let diff = self.repo.diff_tree_to_index(Some(&head), None, None)?;
        Ok(diff.deltas().count() > 0)
    }

    fn has_unstaged_changes(&self) -> Result<bool> {
        let diff = self.repo.diff_index_to_workdir(None, None)?;
        Ok(diff.deltas().count() > 0)
    }

    fn is_clean(&self) -> Result<bool> {
        Ok(!self.has_staged_changes()? && !self.has_unstaged_changes()?)
    }
}

impl GitOps {
    fn get_signature(&self) -> Result<Signature<'_>> {
        self.repo.signature().or_else(|_| {
            // Fallback signature for automation
            Signature::now("Codemod Bot", "codemod@automated.local").map_err(|e| e.into())
        })
    }

    fn commit_internal(
        &self,
        message: &str,
        author: &Signature<'_>,
        committer: &Signature<'_>,
    ) -> Result<git2::Oid> {
        let mut index = self.repo.index()?;
        let tree_id = index.write_tree()?;
        let tree = self.repo.find_tree(tree_id)?;

        // Get parent commit (HEAD)
        let parent = self.repo.head()?.peel_to_commit()?;

        let oid = self
            .repo
            .commit(Some("HEAD"), author, committer, message, &tree, &[&parent])?;

        Ok(oid)
    }
}
