//! File-level transformations (move, rename, delete).

use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};

/// A file operation to be executed.
#[derive(Debug, Clone)]
pub enum FileOperation {
    Move { from: PathBuf, to: PathBuf },
    Copy { from: PathBuf, to: PathBuf },
    Delete { path: PathBuf },
    CreateDir { path: PathBuf },
    Rename { from: PathBuf, to: PathBuf },
}

impl FileOperation {
    /// Executes the file operation.
    pub fn execute(&self) -> Result<()> {
        match self {
            FileOperation::Move { from, to } => {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::rename(from, to)?;
            }
            FileOperation::Copy { from, to } => {
                if let Some(parent) = to.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(from, to)?;
            }
            FileOperation::Delete { path } => {
                if path.is_dir() {
                    fs::remove_dir_all(path)?;
                } else {
                    fs::remove_file(path)?;
                }
            }
            FileOperation::CreateDir { path } => {
                fs::create_dir_all(path)?;
            }
            FileOperation::Rename { from, to } => {
                fs::rename(from, to)?;
            }
        }
        Ok(())
    }

    /// Returns a description of the operation.
    pub fn describe(&self) -> String {
        match self {
            FileOperation::Move { from, to } => {
                format!("Move {} -> {}", from.display(), to.display())
            }
            FileOperation::Copy { from, to } => {
                format!("Copy {} -> {}", from.display(), to.display())
            }
            FileOperation::Delete { path } => {
                format!("Delete {}", path.display())
            }
            FileOperation::CreateDir { path } => {
                format!("Create directory {}", path.display())
            }
            FileOperation::Rename { from, to } => {
                format!("Rename {} -> {}", from.display(), to.display())
            }
        }
    }
}

/// Builder for file transformations.
#[derive(Default)]
pub struct FileTransform {
    operations: Vec<FileOperation>,
}

impl FileTransform {
    /// Creates a new file transform builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves a file or directory to a new location.
    pub fn move_file(mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        self.operations.push(FileOperation::Move {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Copies a file or directory to a new location.
    pub fn copy(mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        self.operations.push(FileOperation::Copy {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Deletes a file or directory.
    pub fn delete(mut self, path: impl Into<PathBuf>) -> Self {
        self.operations.push(FileOperation::Delete { path: path.into() });
        self
    }

    /// Creates a directory (and parent directories).
    pub fn create_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.operations.push(FileOperation::CreateDir { path: path.into() });
        self
    }

    /// Renames a file or directory.
    pub fn rename(mut self, from: impl Into<PathBuf>, to: impl Into<PathBuf>) -> Self {
        self.operations.push(FileOperation::Rename {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Adds a batch of moves following a pattern.
    /// The mapper function receives each source path and returns the destination.
    pub fn move_matching<F>(mut self, paths: impl IntoIterator<Item = PathBuf>, mapper: F) -> Self
    where
        F: Fn(&Path) -> PathBuf,
    {
        for path in paths {
            let dest = mapper(&path);
            self.operations.push(FileOperation::Move {
                from: path,
                to: dest,
            });
        }
        self
    }

    /// Executes all file operations.
    pub fn execute(&self) -> Result<Vec<FileOperation>> {
        for op in &self.operations {
            op.execute()?;
        }
        Ok(self.operations.clone())
    }

    /// Returns descriptions of all operations.
    pub fn describe(&self) -> Vec<String> {
        self.operations.iter().map(|op| op.describe()).collect()
    }

    /// Returns the operations without executing them.
    pub fn operations(&self) -> &[FileOperation] {
        &self.operations
    }

    /// Returns true if there are no operations.
    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }
}
