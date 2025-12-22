//! Error types for the refactor DSL.

use std::path::PathBuf;
use thiserror::Error;

/// The main error type for refactoring operations.
#[derive(Error, Debug)]
pub enum RefactorError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    #[error("Glob pattern error: {0}")]
    Glob(#[from] globset::Error),

    #[error("Tree-sitter parse error for {path}: {message}")]
    Parse { path: PathBuf, message: String },

    #[error("Tree-sitter query error: {0}")]
    Query(#[from] tree_sitter::QueryError),

    #[error("Language not supported: {0}")]
    UnsupportedLanguage(String),

    #[error("No files matched the specified criteria")]
    NoFilesMatched,

    #[error("Repository not found at path: {0}")]
    RepoNotFound(PathBuf),

    #[error("Transform failed: {message}")]
    TransformFailed { message: String },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
}

/// A specialized Result type for refactoring operations.
pub type Result<T> = std::result::Result<T, RefactorError>;
