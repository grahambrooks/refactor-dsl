//! AST-aware code transformations using tree-sitter.

use super::Transform;
use crate::error::{RefactorError, Result};
use crate::lang::LanguageRegistry;
use crate::matcher::ast::AstMatch;
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

/// AST-aware transformation builder.
pub struct AstTransform {
    query: Option<String>,
    operations: Vec<AstOperation>,
    registry: LanguageRegistry,
}

/// An operation to perform on matched AST nodes.
#[derive(Clone)]
enum AstOperation {
    Replace { replacement: String },
    ReplaceWith { template: String },
    Rename { from: String, to: String },
    Wrap { prefix: String, suffix: String },
    Delete,
}

impl AstTransform {
    /// Creates a new AST transform builder.
    pub fn new() -> Self {
        Self {
            query: None,
            operations: Vec::new(),
            registry: LanguageRegistry::new(),
        }
    }

    /// Sets the tree-sitter query pattern.
    pub fn query(mut self, pattern: impl Into<String>) -> Self {
        self.query = Some(pattern.into());
        self
    }

    /// Replaces matched nodes with the given text.
    pub fn replace(mut self, replacement: impl Into<String>) -> Self {
        self.operations.push(AstOperation::Replace {
            replacement: replacement.into(),
        });
        self
    }

    /// Replaces matched nodes using a template with capture references.
    /// Use @capture_name to reference captures from the query.
    pub fn replace_with(mut self, template: impl Into<String>) -> Self {
        self.operations.push(AstOperation::ReplaceWith {
            template: template.into(),
        });
        self
    }

    /// Renames identifiers from one name to another.
    pub fn rename(mut self, from: impl Into<String>, to: impl Into<String>) -> Self {
        self.operations.push(AstOperation::Rename {
            from: from.into(),
            to: to.into(),
        });
        self
    }

    /// Wraps matched nodes with prefix and suffix.
    pub fn wrap(mut self, prefix: impl Into<String>, suffix: impl Into<String>) -> Self {
        self.operations.push(AstOperation::Wrap {
            prefix: prefix.into(),
            suffix: suffix.into(),
        });
        self
    }

    /// Deletes matched nodes.
    pub fn delete(mut self) -> Self {
        self.operations.push(AstOperation::Delete);
        self
    }

    /// Finds all matches for the query in the source.
    fn find_matches(&self, source: &str, path: &Path) -> Result<Vec<AstMatch>> {
        let Some(ref query_str) = self.query else {
            return Ok(Vec::new());
        };

        let lang = self
            .registry
            .detect(path)
            .ok_or_else(|| RefactorError::UnsupportedLanguage(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ))?;

        let tree = lang.parse(source)?;
        let query = lang.query(query_str)?;
        let mut cursor = QueryCursor::new();
        let source_bytes = source.as_bytes();
        let mut matches = Vec::new();

        let mut query_matches = cursor.matches(&query, tree.root_node(), source_bytes);
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

        // Sort by start position in reverse order for safe replacement
        matches.sort_by(|a, b| b.start_byte.cmp(&a.start_byte));
        Ok(matches)
    }

    /// Applies a single operation to the source at the given match.
    fn apply_operation(
        &self,
        source: &mut String,
        match_info: &AstMatch,
        operation: &AstOperation,
    ) -> Result<()> {
        match operation {
            AstOperation::Replace { replacement } => {
                source.replace_range(match_info.start_byte..match_info.end_byte, replacement);
            }
            AstOperation::ReplaceWith { template } => {
                // Simple template expansion: replace @capture with the matched text
                let expanded = template.replace(&format!("@{}", match_info.capture_name), &match_info.text);
                source.replace_range(match_info.start_byte..match_info.end_byte, &expanded);
            }
            AstOperation::Rename { from, to } => {
                if match_info.text == *from {
                    source.replace_range(match_info.start_byte..match_info.end_byte, to);
                }
            }
            AstOperation::Wrap { prefix, suffix } => {
                let wrapped = format!("{}{}{}", prefix, match_info.text, suffix);
                source.replace_range(match_info.start_byte..match_info.end_byte, &wrapped);
            }
            AstOperation::Delete => {
                source.replace_range(match_info.start_byte..match_info.end_byte, "");
            }
        }
        Ok(())
    }
}

impl Default for AstTransform {
    fn default() -> Self {
        Self::new()
    }
}

impl Transform for AstTransform {
    fn apply(&self, source: &str, path: &Path) -> Result<String> {
        if self.query.is_none() || self.operations.is_empty() {
            return Ok(source.to_string());
        }

        let matches = self.find_matches(source, path)?;
        let mut result = source.to_string();

        for match_info in matches {
            for operation in &self.operations {
                self.apply_operation(&mut result, &match_info, operation)?;
            }
        }

        Ok(result)
    }

    fn describe(&self) -> String {
        let query_desc = self.query.as_deref().unwrap_or("(no query)");
        let ops: Vec<String> = self.operations.iter().map(|op| {
            match op {
                AstOperation::Replace { replacement } => format!("replace with '{}'", replacement),
                AstOperation::ReplaceWith { template } => format!("replace using template '{}'", template),
                AstOperation::Rename { from, to } => format!("rename '{}' to '{}'", from, to),
                AstOperation::Wrap { prefix, suffix } => format!("wrap with '{}' and '{}'", prefix, suffix),
                AstOperation::Delete => "delete".to_string(),
            }
        }).collect();
        format!("AST query '{}': {}", query_desc, ops.join(", "))
    }
}
