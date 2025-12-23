//! Transform DSL for code refactoring operations.

pub mod ast;
pub mod file;
pub mod text;

pub use ast::AstTransform;
pub use file::FileTransform;
pub use text::TextTransform;

use crate::error::Result;
use std::path::Path;

/// A code transformation that can be applied to source files.
pub trait Transform: Send + Sync {
    /// Applies the transformation to the given source code.
    fn apply(&self, source: &str, path: &Path) -> Result<String>;

    /// Returns a description of the transformation.
    fn describe(&self) -> String;
}

/// The main transform builder that combines multiple transformations.
#[derive(Default)]
pub struct TransformBuilder {
    transforms: Vec<Box<dyn Transform>>,
}

impl TransformBuilder {
    /// Creates a new transform builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a text-based pattern replacement.
    pub fn replace_pattern(mut self, pattern: &str, replacement: &str) -> Self {
        self.transforms
            .push(Box::new(TextTransform::replace(pattern, replacement)));
        self
    }

    /// Adds a literal string replacement.
    pub fn replace_literal(mut self, needle: &str, replacement: &str) -> Self {
        self.transforms
            .push(Box::new(TextTransform::replace_literal(
                needle,
                replacement,
            )));
        self
    }

    /// Adds an AST-based transformation.
    pub fn ast<F>(mut self, f: F) -> Self
    where
        F: FnOnce(AstTransform) -> AstTransform,
    {
        self.transforms.push(Box::new(f(AstTransform::new())));
        self
    }

    /// Adds a custom transformation.
    pub fn custom<T: Transform + 'static>(mut self, transform: T) -> Self {
        self.transforms.push(Box::new(transform));
        self
    }

    /// Applies all transformations to the source code in order.
    pub fn apply(&self, source: &str, path: &Path) -> Result<String> {
        let mut result = source.to_string();
        for transform in &self.transforms {
            result = transform.apply(&result, path)?;
        }
        Ok(result)
    }

    /// Returns descriptions of all transformations.
    pub fn describe(&self) -> Vec<String> {
        self.transforms.iter().map(|t| t.describe()).collect()
    }

    /// Returns the number of transformations.
    pub fn len(&self) -> usize {
        self.transforms.len()
    }

    /// Returns true if there are no transformations.
    pub fn is_empty(&self) -> bool {
        self.transforms.is_empty()
    }
}

/// Represents a change to be applied to a file.
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: std::path::PathBuf,
    pub original: String,
    pub transformed: String,
}

impl FileChange {
    /// Returns true if the content was modified.
    pub fn is_modified(&self) -> bool {
        self.original != self.transformed
    }

    /// Writes the transformed content to disk.
    pub fn apply(&self) -> Result<()> {
        if self.is_modified() {
            std::fs::write(&self.path, &self.transformed)?;
        }
        Ok(())
    }
}
