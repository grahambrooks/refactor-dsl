//! AST-based code matching using tree-sitter queries.

use crate::error::Result;
use crate::lang::{Language, LanguageRegistry};
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor};

/// Match result containing the matched text and its location.
#[derive(Debug, Clone)]
pub struct AstMatch {
    pub text: String,
    pub start_byte: usize,
    pub end_byte: usize,
    pub start_row: usize,
    pub start_col: usize,
    pub end_row: usize,
    pub end_col: usize,
    pub capture_name: String,
}

/// AST-based matching using tree-sitter queries.
#[derive(Default, Clone)]
pub struct AstMatcher {
    queries: Vec<String>,
    capture_names: Vec<String>,
}

impl AstMatcher {
    /// Creates a new AST matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a tree-sitter query pattern.
    ///
    /// Query syntax follows tree-sitter's S-expression format:
    /// - `(function_item name: (identifier) @fn_name)` - captures function names
    /// - `(call_expression function: (identifier) @fn)` - captures function calls
    pub fn query(mut self, pattern: impl Into<String>) -> Self {
        self.queries.push(pattern.into());
        self
    }

    /// Filters matches to only those with the specified capture name.
    pub fn capture(mut self, name: impl Into<String>) -> Self {
        self.capture_names.push(name.into());
        self
    }

    /// Finds all matches in the given source code.
    pub fn find_matches(&self, source: &str, lang: &dyn Language) -> Result<Vec<AstMatch>> {
        let tree = lang.parse(source)?;
        let mut all_matches = Vec::new();

        for query_str in &self.queries {
            let query = lang.query(query_str)?;
            let matches = self.execute_query(&query, &tree, source)?;
            all_matches.extend(matches);
        }

        // Filter by capture names if specified
        if !self.capture_names.is_empty() {
            all_matches.retain(|m| self.capture_names.contains(&m.capture_name));
        }

        Ok(all_matches)
    }

    /// Finds all matches in a file, auto-detecting the language.
    pub fn find_matches_in_file(
        &self,
        path: &Path,
        registry: &LanguageRegistry,
    ) -> Result<Vec<AstMatch>> {
        let source = std::fs::read_to_string(path)?;
        let lang = registry.detect(path).ok_or_else(|| {
            crate::error::RefactorError::UnsupportedLanguage(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        self.find_matches(&source, lang)
    }

    fn execute_query(
        &self,
        query: &Query,
        tree: &tree_sitter::Tree,
        source: &str,
    ) -> Result<Vec<AstMatch>> {
        let mut cursor = QueryCursor::new();
        let source_bytes = source.as_bytes();
        let mut matches = Vec::new();

        let mut query_matches = cursor.matches(query, tree.root_node(), source_bytes);
        while let Some(query_match) = query_matches.next() {
            for capture in query_match.captures {
                let node = capture.node;
                let capture_name = query.capture_names()[capture.index as usize].to_string();
                let text = node.utf8_text(source_bytes).unwrap_or("").to_string();

                matches.push(AstMatch {
                    text,
                    start_byte: node.start_byte(),
                    end_byte: node.end_byte(),
                    start_row: node.start_position().row,
                    start_col: node.start_position().column,
                    end_row: node.end_position().row,
                    end_col: node.end_position().column,
                    capture_name,
                });
            }
        }

        Ok(matches)
    }

    /// Returns true if the source contains any matches.
    pub fn has_matches(&self, source: &str, lang: &dyn Language) -> Result<bool> {
        Ok(!self.find_matches(source, lang)?.is_empty())
    }

    /// Returns the query strings.
    pub fn queries(&self) -> &[String] {
        &self.queries
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lang::{Python, Rust, TypeScript};

    #[test]
    fn test_find_rust_functions() {
        let source = r#"
fn hello() {}
fn world() {}
pub fn greet(name: &str) {}
"#;
        let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn_name)");

        let matches = matcher.find_matches(source, &Rust).unwrap();

        assert_eq!(matches.len(), 3);
        let names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"world"));
        assert!(names.contains(&"greet"));
    }

    #[test]
    fn test_find_rust_structs() {
        let source = r#"
struct Point { x: i32, y: i32 }
struct Circle { center: Point, radius: f64 }
"#;
        let matcher = AstMatcher::new().query("(struct_item name: (type_identifier) @struct_name)");

        let matches = matcher.find_matches(source, &Rust).unwrap();

        assert_eq!(matches.len(), 2);
        let names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(names.contains(&"Point"));
        assert!(names.contains(&"Circle"));
    }

    #[test]
    fn test_find_typescript_functions() {
        let source = r#"
function hello() { }
function world(): void { }
"#;
        let matcher = AstMatcher::new().query("(function_declaration name: (identifier) @fn_name)");

        let matches = matcher.find_matches(source, &TypeScript).unwrap();

        assert_eq!(matches.len(), 2);
        let names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"world"));
    }

    #[test]
    fn test_find_python_functions() {
        let source = r#"
def hello():
    pass

def world(x):
    return x * 2
"#;
        let matcher = AstMatcher::new().query("(function_definition name: (identifier) @fn_name)");

        let matches = matcher.find_matches(source, &Python).unwrap();

        assert_eq!(matches.len(), 2);
        let names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"world"));
    }

    #[test]
    fn test_capture_filtering() {
        let source = r#"
fn process(data: Vec<u8>) -> Result<String> {
    let x = 42;
    Ok(format!("{}", x))
}
"#;
        // Query captures both function name and parameter name
        let matcher = AstMatcher::new()
            .query("(function_item name: (identifier) @fn_name)")
            .query("(parameter pattern: (identifier) @param)")
            .capture("fn_name");

        let matches = matcher.find_matches(source, &Rust).unwrap();

        // Should only have the function name, not the parameter
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "process");
        assert_eq!(matches[0].capture_name, "fn_name");
    }

    #[test]
    fn test_match_positions() {
        let source = "fn test() {}";
        let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn_name)");

        let matches = matcher.find_matches(source, &Rust).unwrap();

        assert_eq!(matches.len(), 1);
        let m = &matches[0];
        assert_eq!(m.text, "test");
        assert_eq!(m.start_row, 0);
        assert_eq!(m.start_col, 3);
        assert_eq!(m.start_byte, 3);
        assert_eq!(m.end_byte, 7);
    }

    #[test]
    fn test_has_matches() {
        let source = "fn hello() {}";
        let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn)");

        assert!(matcher.has_matches(source, &Rust).unwrap());

        let no_fn_source = "let x = 42;";
        assert!(!matcher.has_matches(no_fn_source, &Rust).unwrap());
    }

    #[test]
    fn test_multiple_queries() {
        let source = r#"
fn hello() {}
struct Point { x: i32 }
"#;
        let matcher = AstMatcher::new()
            .query("(function_item name: (identifier) @fn)")
            .query("(struct_item name: (type_identifier) @struct)");

        let matches = matcher.find_matches(source, &Rust).unwrap();

        assert_eq!(matches.len(), 2);
        let names: Vec<&str> = matches.iter().map(|m| m.text.as_str()).collect();
        assert!(names.contains(&"hello"));
        assert!(names.contains(&"Point"));
    }

    #[test]
    fn test_no_matches() {
        let source = "let x = 42;";
        let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn)");

        let matches = matcher.find_matches(source, &Rust).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_empty_source() {
        let source = "";
        let matcher = AstMatcher::new().query("(function_item name: (identifier) @fn)");

        let matches = matcher.find_matches(source, &Rust).unwrap();
        assert!(matches.is_empty());
    }

    #[test]
    fn test_queries_getter() {
        let matcher = AstMatcher::new()
            .query("(function_item @fn)")
            .query("(struct_item @struct)");

        let queries = matcher.queries();
        assert_eq!(queries.len(), 2);
    }
}
