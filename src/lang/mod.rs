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

        parser
            .parse(source, None)
            .ok_or_else(|| RefactorError::Parse {
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
        self.extensions()
            .iter()
            .any(|e| e.eq_ignore_ascii_case(ext))
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_language() {
        let rust = Rust;
        assert_eq!(rust.name(), "rust");
        assert!(rust.extensions().contains(&"rs"));
        assert!(rust.matches_extension("rs"));
        assert!(rust.matches_extension("RS")); // case insensitive
        assert!(!rust.matches_extension("py"));
    }

    #[test]
    fn test_typescript_language() {
        let ts = TypeScript;
        assert_eq!(ts.name(), "typescript");
        assert!(ts.matches_extension("ts"));
        assert!(ts.matches_extension("tsx"));
        assert!(ts.matches_extension("js"));
        assert!(ts.matches_extension("jsx"));
    }

    #[test]
    fn test_python_language() {
        let py = Python;
        assert_eq!(py.name(), "python");
        assert!(py.matches_extension("py"));
        assert!(py.matches_extension("pyi"));
    }

    #[test]
    fn test_rust_parsing() {
        let rust = Rust;
        let source = "fn main() { println!(\"Hello\"); }";
        let tree = rust.parse(source).expect("Failed to parse");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_typescript_parsing() {
        let ts = TypeScript;
        let source = "function hello(): void { console.log('hi'); }";
        let tree = ts.parse(source).expect("Failed to parse");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_python_parsing() {
        let py = Python;
        let source = "def hello():\n    print('hi')";
        let tree = py.parse(source).expect("Failed to parse");
        assert!(!tree.root_node().has_error());
    }

    #[test]
    fn test_rust_query() {
        let rust = Rust;
        let query = rust.query("(function_item name: (identifier) @fn)");
        assert!(query.is_ok());
    }

    #[test]
    fn test_invalid_query() {
        let rust = Rust;
        let query = rust.query("(invalid_node_type @capture)");
        assert!(query.is_err());
    }

    #[test]
    fn test_registry_new() {
        let registry = LanguageRegistry::new();
        assert_eq!(registry.all().len(), 3);
    }

    #[test]
    fn test_registry_by_extension() {
        let registry = LanguageRegistry::new();

        let rust = registry.by_extension("rs");
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().name(), "rust");

        let ts = registry.by_extension("ts");
        assert!(ts.is_some());
        assert_eq!(ts.unwrap().name(), "typescript");

        let py = registry.by_extension("py");
        assert!(py.is_some());
        assert_eq!(py.unwrap().name(), "python");

        let unknown = registry.by_extension("xyz");
        assert!(unknown.is_none());
    }

    #[test]
    fn test_registry_by_name() {
        let registry = LanguageRegistry::new();

        assert!(registry.by_name("rust").is_some());
        assert!(registry.by_name("RUST").is_some()); // case insensitive
        assert!(registry.by_name("typescript").is_some());
        assert!(registry.by_name("python").is_some());
        assert!(registry.by_name("cobol").is_none());
    }

    #[test]
    fn test_registry_detect() {
        let registry = LanguageRegistry::new();

        let rust = registry.detect(Path::new("src/main.rs"));
        assert!(rust.is_some());
        assert_eq!(rust.unwrap().name(), "rust");

        let ts = registry.detect(Path::new("app/index.tsx"));
        assert!(ts.is_some());
        assert_eq!(ts.unwrap().name(), "typescript");

        let no_ext = registry.detect(Path::new("Makefile"));
        assert!(no_ext.is_none());
    }
}
