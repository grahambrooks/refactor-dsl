//! Language abstraction for multi-language parsing and refactoring.

mod python;
mod rust;
mod typescript;

pub use python::Python;
pub use rust::Rust;
pub use typescript::TypeScript;

use crate::error::{RefactorError, Result};
use std::path::Path;
use tree_sitter::{Language as TsLanguage, Parser, Query, Tree};

/// A programming language supported by the refactoring DSL.
pub trait Language: Send + Sync {
    /// Returns the name of the language.
    fn name(&self) -> &'static str;

    /// Returns the file extensions associated with this language.
    fn extensions(&self) -> &[&'static str];

    /// Returns the tree-sitter language grammar.
    fn grammar(&self) -> TsLanguage;

    /// Parses source code into a tree-sitter AST.
    fn parse(&self, source: &str) -> Result<Tree> {
        let mut parser = Parser::new();
        parser
            .set_language(&self.grammar())
            .map_err(|e| RefactorError::Parse {
                path: Path::new("<source>").to_path_buf(),
                message: format!("Failed to set language: {e}"),
            })?;

        parser.parse(source, None).ok_or_else(|| RefactorError::Parse {
            path: Path::new("<source>").to_path_buf(),
            message: "Failed to parse source".to_string(),
        })
    }

    /// Creates a tree-sitter query for this language.
    fn query(&self, pattern: &str) -> Result<Query> {
        Ok(Query::new(&self.grammar(), pattern)?)
    }

    /// Checks if this language handles the given file extension.
    fn matches_extension(&self, ext: &str) -> bool {
        self.extensions().iter().any(|e| e.eq_ignore_ascii_case(ext))
    }
}

/// Registry of supported languages.
#[derive(Default)]
pub struct LanguageRegistry {
    languages: Vec<Box<dyn Language>>,
}

impl LanguageRegistry {
    /// Creates a new registry with all built-in languages.
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register(Box::new(Rust));
        registry.register(Box::new(TypeScript));
        registry.register(Box::new(Python));
        registry
    }

    /// Registers a new language.
    pub fn register(&mut self, lang: Box<dyn Language>) {
        self.languages.push(lang);
    }

    /// Finds a language by file extension.
    pub fn by_extension(&self, ext: &str) -> Option<&dyn Language> {
        self.languages
            .iter()
            .find(|l| l.matches_extension(ext))
            .map(|l| l.as_ref())
    }

    /// Finds a language by name.
    pub fn by_name(&self, name: &str) -> Option<&dyn Language> {
        self.languages
            .iter()
            .find(|l| l.name().eq_ignore_ascii_case(name))
            .map(|l| l.as_ref())
    }

    /// Detects the language for a given file path.
    pub fn detect(&self, path: &Path) -> Option<&dyn Language> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.by_extension(ext))
    }

    /// Returns all registered languages.
    pub fn all(&self) -> &[Box<dyn Language>] {
        &self.languages
    }
}
