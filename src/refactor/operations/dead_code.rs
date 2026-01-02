//! Dead code detection and analysis.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};
use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::{RefactorError, Result};
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};
use super::RefactoringOperation;

/// Find and report dead code in a codebase.
#[derive(Debug, Clone)]
pub struct FindDeadCode {
    /// Types of dead code to search for.
    pub include_types: HashSet<DeadCodeType>,
    /// Whether to search recursively in directories.
    pub recursive: bool,
    /// Files to exclude.
    pub exclude_patterns: Vec<String>,
    /// Whether to auto-delete found dead code.
    pub auto_delete: bool,
    /// Additional search paths.
    pub search_paths: Vec<PathBuf>,
}

/// Types of dead code to detect.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeadCodeType {
    /// Unused functions/methods.
    UnusedFunctions,
    /// Unused variables.
    UnusedVariables,
    /// Unused imports.
    UnusedImports,
    /// Unused types (structs, classes, etc.).
    UnusedTypes,
    /// Unused constants.
    UnusedConstants,
    /// Unreachable code.
    UnreachableCode,
    /// Empty blocks or functions.
    EmptyBlocks,
    /// Commented-out code.
    CommentedCode,
}

impl Default for FindDeadCode {
    fn default() -> Self {
        Self::new()
    }
}

impl FindDeadCode {
    /// Create a new FindDeadCode operation with default settings.
    pub fn new() -> Self {
        let mut include_types = HashSet::new();
        include_types.insert(DeadCodeType::UnusedFunctions);
        include_types.insert(DeadCodeType::UnusedVariables);
        include_types.insert(DeadCodeType::UnusedImports);

        Self {
            include_types,
            recursive: true,
            exclude_patterns: Vec::new(),
            auto_delete: false,
            search_paths: Vec::new(),
        }
    }

    /// Include a specific type of dead code to search for.
    pub fn include(mut self, code_type: DeadCodeType) -> Self {
        self.include_types.insert(code_type);
        self
    }

    /// Exclude a specific type of dead code.
    pub fn exclude(mut self, code_type: DeadCodeType) -> Self {
        self.include_types.remove(&code_type);
        self
    }

    /// Set to search only specified types.
    pub fn only(mut self, types: Vec<DeadCodeType>) -> Self {
        self.include_types.clear();
        for t in types {
            self.include_types.insert(t);
        }
        self
    }

    /// Add an exclude pattern.
    pub fn exclude_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_patterns.push(pattern.into());
        self
    }

    /// Enable auto-delete of found dead code.
    pub fn auto_delete(mut self) -> Self {
        self.auto_delete = true;
        self
    }

    /// Add a search path.
    pub fn search_in(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Disable recursive search.
    pub fn no_recursive(mut self) -> Self {
        self.recursive = false;
        self
    }

    /// Find all definitions in the source.
    fn find_definitions(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Vec<Definition>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let mut definitions = Vec::new();

        // Find functions
        if self.include_types.contains(&DeadCodeType::UnusedFunctions) {
            definitions.extend(self.find_functions(ctx, lang, &tree, source_bytes)?);
        }

        // Find variables
        if self.include_types.contains(&DeadCodeType::UnusedVariables) {
            definitions.extend(self.find_variables(ctx, lang, &tree, source_bytes)?);
        }

        // Find imports
        if self.include_types.contains(&DeadCodeType::UnusedImports) {
            definitions.extend(self.find_imports(ctx, lang, &tree, source_bytes)?);
        }

        // Find types
        if self.include_types.contains(&DeadCodeType::UnusedTypes) {
            definitions.extend(self.find_types(ctx, lang, &tree, source_bytes)?);
        }

        // Find constants
        if self.include_types.contains(&DeadCodeType::UnusedConstants) {
            definitions.extend(self.find_constants(ctx, lang, &tree, source_bytes)?);
        }

        Ok(definitions)
    }

    /// Find function definitions.
    fn find_functions(
        &self,
        _ctx: &RefactoringContext,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
    ) -> Result<Vec<Definition>> {
        let query_str = match lang.name() {
            "rust" => "(function_item name: (identifier) @name) @def",
            "typescript" | "javascript" => "(function_declaration name: (identifier) @name) @def",
            "python" => "(function_definition name: (identifier) @name) @def",
            "go" => "(function_declaration name: (identifier) @name) @def",
            "java" | "csharp" => "(method_declaration name: (identifier) @name) @def",
            "ruby" => "(method name: (identifier) @name) @def",
            _ => return Ok(Vec::new()),
        };

        self.query_definitions(lang, tree, source_bytes, query_str, DefinitionKind::Function)
    }

    /// Find variable definitions.
    fn find_variables(
        &self,
        _ctx: &RefactoringContext,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
    ) -> Result<Vec<Definition>> {
        let query_str = match lang.name() {
            "rust" => "(let_declaration pattern: (identifier) @name) @def",
            "typescript" | "javascript" => "(variable_declarator name: (identifier) @name) @def",
            "python" => "(assignment left: (identifier) @name) @def",
            "go" => "(short_var_declaration left: (expression_list (identifier) @name)) @def",
            _ => return Ok(Vec::new()),
        };

        self.query_definitions(lang, tree, source_bytes, query_str, DefinitionKind::Variable)
    }

    /// Find import definitions.
    fn find_imports(
        &self,
        _ctx: &RefactoringContext,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
    ) -> Result<Vec<Definition>> {
        let query_str = match lang.name() {
            "rust" => "(use_declaration argument: (use_tree) @name) @def",
            "typescript" | "javascript" => {
                "(import_specifier name: (identifier) @name) @def"
            }
            "python" => "(import_from_statement name: (dotted_name) @name) @def",
            "go" => "(import_spec path: (interpreted_string_literal) @name) @def",
            "java" => "(import_declaration (scoped_identifier) @name) @def",
            _ => return Ok(Vec::new()),
        };

        self.query_definitions(lang, tree, source_bytes, query_str, DefinitionKind::Import)
    }

    /// Find type definitions.
    fn find_types(
        &self,
        _ctx: &RefactoringContext,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
    ) -> Result<Vec<Definition>> {
        let query_str = match lang.name() {
            "rust" => {
                r#"
                (struct_item name: (type_identifier) @name) @def
                (enum_item name: (type_identifier) @name) @def
                "#
            }
            "typescript" => {
                r#"
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                (type_alias_declaration name: (type_identifier) @name) @def
                "#
            }
            "python" => "(class_definition name: (identifier) @name) @def",
            "go" => "(type_declaration (type_spec name: (type_identifier) @name)) @def",
            "java" | "csharp" => {
                r#"
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                "#
            }
            _ => return Ok(Vec::new()),
        };

        self.query_definitions(lang, tree, source_bytes, query_str, DefinitionKind::Type)
    }

    /// Find constant definitions.
    fn find_constants(
        &self,
        _ctx: &RefactoringContext,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
    ) -> Result<Vec<Definition>> {
        let query_str = match lang.name() {
            "rust" => "(const_item name: (identifier) @name) @def",
            "typescript" | "javascript" => {
                "(lexical_declaration (variable_declarator name: (identifier) @name)) @def"
            }
            "go" => "(const_declaration (const_spec name: (identifier) @name)) @def",
            "java" => "(field_declaration (variable_declarator name: (identifier) @name)) @def",
            _ => return Ok(Vec::new()),
        };

        self.query_definitions(lang, tree, source_bytes, query_str, DefinitionKind::Constant)
    }

    /// Execute a query and extract definitions.
    fn query_definitions(
        &self,
        lang: &dyn Language,
        tree: &tree_sitter::Tree,
        source_bytes: &[u8],
        query_str: &str,
        kind: DefinitionKind,
    ) -> Result<Vec<Definition>> {
        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut definitions = Vec::new();

        while let Some(m) = matches.next() {
            let mut name = None;
            let mut def_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "def" => {
                        let node = capture.node;
                        def_range = Some(Range {
                            start: Position {
                                line: node.start_position().row as u32,
                                character: node.start_position().column as u32,
                            },
                            end: Position {
                                line: node.end_position().row as u32,
                                character: node.end_position().column as u32,
                            },
                        });
                    }
                    _ => {}
                }
            }

            if let (Some(n), Some(r)) = (name, def_range) {
                // Skip if name starts with underscore (conventionally unused)
                if !n.starts_with('_') {
                    definitions.push(Definition {
                        name: n.to_string(),
                        range: r,
                        kind,
                    });
                }
            }
        }

        Ok(definitions)
    }

    /// Count usages of a symbol.
    fn count_usages(
        &self,
        source: &str,
        lang: &dyn Language,
        name: &str,
        def_range: &Range,
    ) -> Result<usize> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        let query = lang.query("(identifier) @id")?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut count = 0;

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if let Ok(text) = capture.node.utf8_text(source_bytes) {
                    if text == name {
                        let node = capture.node;
                        let range = Range {
                            start: Position {
                                line: node.start_position().row as u32,
                                character: node.start_position().column as u32,
                            },
                            end: Position {
                                line: node.end_position().row as u32,
                                character: node.end_position().column as u32,
                            },
                        };

                        // Skip the definition itself
                        let is_definition = range.start.line == def_range.start.line
                            && range.start.character >= def_range.start.character
                            && range.end.character <= def_range.end.character;

                        if !is_definition {
                            count += 1;
                        }
                    }
                }
            }
        }

        Ok(count)
    }

    /// Find empty blocks.
    fn find_empty_blocks(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Vec<DeadCodeItem>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => "(block) @block",
            "typescript" | "javascript" => "(statement_block) @block",
            "python" => "(block) @block",
            _ => return Ok(Vec::new()),
        };

        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut items = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if let Ok(text) = capture.node.utf8_text(source_bytes) {
                    let inner = text.trim();
                    let inner = if inner.starts_with('{') && inner.ends_with('}') {
                        inner[1..inner.len() - 1].trim()
                    } else {
                        inner
                    };

                    // Check if block is empty or only has pass/comments
                    let is_empty = inner.is_empty()
                        || inner == "pass"
                        || inner.lines().all(|l| {
                            let t = l.trim();
                            t.is_empty() || t.starts_with("//") || t.starts_with('#')
                        });

                    if is_empty && !inner.is_empty() {
                        let node = capture.node;
                        items.push(DeadCodeItem {
                            name: "empty block".to_string(),
                            code_type: DeadCodeType::EmptyBlocks,
                            range: Range {
                                start: Position {
                                    line: node.start_position().row as u32,
                                    character: node.start_position().column as u32,
                                },
                                end: Position {
                                    line: node.end_position().row as u32,
                                    character: node.end_position().column as u32,
                                },
                            },
                            context: text.chars().take(50).collect(),
                        });
                    }
                }
            }
        }

        Ok(items)
    }

    /// Find commented-out code.
    fn find_commented_code(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Vec<DeadCodeItem>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" | "go" | "java" | "csharp" | "typescript" | "javascript" => {
                "(comment) @comment"
            }
            "python" | "ruby" => "(comment) @comment",
            _ => return Ok(Vec::new()),
        };

        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut items = Vec::new();

        // Patterns that suggest commented-out code
        let code_patterns = [
            "fn ", "let ", "const ", "var ", "if ", "for ", "while ", "return ",
            "function ", "class ", "def ", "import ", "from ", "struct ", "enum ",
        ];

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if let Ok(text) = capture.node.utf8_text(source_bytes) {
                    let content = text
                        .trim_start_matches("//")
                        .trim_start_matches("/*")
                        .trim_end_matches("*/")
                        .trim_start_matches('#')
                        .trim();

                    // Check if it looks like code
                    let looks_like_code = code_patterns.iter().any(|p| content.starts_with(p))
                        || (content.contains('(') && content.contains(')'))
                        || (content.contains('{') && content.contains('}'))
                        || content.ends_with(';');

                    if looks_like_code && content.len() > 10 {
                        let node = capture.node;
                        items.push(DeadCodeItem {
                            name: "commented code".to_string(),
                            code_type: DeadCodeType::CommentedCode,
                            range: Range {
                                start: Position {
                                    line: node.start_position().row as u32,
                                    character: node.start_position().column as u32,
                                },
                                end: Position {
                                    line: node.end_position().row as u32,
                                    character: node.end_position().column as u32,
                                },
                            },
                            context: content.chars().take(50).collect(),
                        });
                    }
                }
            }
        }

        Ok(items)
    }
}

/// A definition in the source code.
#[derive(Debug, Clone)]
struct Definition {
    name: String,
    range: Range,
    kind: DefinitionKind,
}

/// Kind of definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DefinitionKind {
    Function,
    Variable,
    Import,
    Type,
    Constant,
}

/// A dead code item found during analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeItem {
    /// Name of the dead code item.
    pub name: String,
    /// Type of dead code.
    pub code_type: DeadCodeType,
    /// Location in the source.
    pub range: Range,
    /// Context snippet.
    pub context: String,
}

/// Result of dead code analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeReport {
    /// File analyzed.
    pub file: PathBuf,
    /// Dead code items found.
    pub items: Vec<DeadCodeItem>,
    /// Summary statistics.
    pub summary: DeadCodeSummary,
}

/// Summary of dead code findings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeadCodeSummary {
    /// Count by type.
    pub by_type: HashMap<DeadCodeType, usize>,
    /// Total items found.
    pub total: usize,
}

impl RefactoringOperation for FindDeadCode {
    fn name(&self) -> &'static str {
        "Find Dead Code"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        if self.include_types.is_empty() {
            return Ok(ValidationResult::invalid(
                "No dead code types selected for analysis",
            ));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let definitions = self.find_definitions(ctx, lang)?;

        // Find unused definitions
        let mut dead_items: Vec<DeadCodeItem> = Vec::new();

        for def in &definitions {
            let usage_count = self.count_usages(&ctx.source, lang, &def.name, &def.range)?;

            if usage_count == 0 {
                let code_type = match def.kind {
                    DefinitionKind::Function => DeadCodeType::UnusedFunctions,
                    DefinitionKind::Variable => DeadCodeType::UnusedVariables,
                    DefinitionKind::Import => DeadCodeType::UnusedImports,
                    DefinitionKind::Type => DeadCodeType::UnusedTypes,
                    DefinitionKind::Constant => DeadCodeType::UnusedConstants,
                };

                dead_items.push(DeadCodeItem {
                    name: def.name.clone(),
                    code_type,
                    range: def.range,
                    context: self.get_context(&ctx.source, def.range.start.line),
                });
            }
        }

        // Find empty blocks
        if self.include_types.contains(&DeadCodeType::EmptyBlocks) {
            dead_items.extend(self.find_empty_blocks(ctx, lang)?);
        }

        // Find commented code
        if self.include_types.contains(&DeadCodeType::CommentedCode) {
            dead_items.extend(self.find_commented_code(ctx, lang)?);
        }

        let mut preview = RefactoringPreview::new(format!(
            "Found {} dead code item(s)",
            dead_items.len()
        ));

        // If auto-delete is enabled, add delete edits
        if self.auto_delete {
            for item in &dead_items {
                let delete_range = Range {
                    start: Position {
                        line: item.range.start.line,
                        character: 0,
                    },
                    end: Position {
                        line: item.range.end.line + 1,
                        character: 0,
                    },
                };

                preview.add_edit(TextEdit::new(
                    ctx.target_file.clone(),
                    delete_range,
                    String::new(),
                ));
            }
        }

        // Build summary
        let mut summary = DeadCodeSummary::default();
        for item in &dead_items {
            *summary.by_type.entry(item.code_type).or_insert(0) += 1;
            summary.total += 1;
        }

        let diff = format!(
            "Dead Code Analysis Report\n{}\nTotal: {} item(s)\n\n{}",
            "=".repeat(40),
            dead_items.len(),
            dead_items
                .iter()
                .map(|item| format!(
                    "[{:?}] {} (line {}): {}",
                    item.code_type,
                    item.name,
                    item.range.start.line + 1,
                    item.context
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );
        preview = preview.with_diff(diff);

        Ok(preview)
    }

    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        let preview = self.preview(ctx)?;

        if !self.auto_delete {
            // Just return the report
            return Ok(RefactoringResult::success(format!(
                "Found {} dead code item(s). Use auto_delete() to remove them.\n\n{}",
                preview.edits.len(),
                preview.diff
            )));
        }

        let mut new_source = ctx.source.clone();

        // Sort edits by position (descending)
        let mut edits = preview.edits.clone();
        edits.sort_by(|a, b| {
            b.range
                .start
                .line
                .cmp(&a.range.start.line)
                .then(b.range.start.character.cmp(&a.range.start.character))
        });

        for edit in &edits {
            let start = ctx.position_to_offset(&edit.range.start);
            let end = ctx.position_to_offset(&edit.range.end);

            if start <= end && end <= new_source.len() {
                new_source.replace_range(start..end, &edit.new_text);
            }
        }

        std::fs::write(&ctx.target_file, &new_source)?;
        ctx.source = new_source;

        Ok(RefactoringResult::success(format!(
            "Deleted {} dead code item(s)",
            edits.len()
        ))
        .with_file(ctx.target_file.clone())
        .with_edits(edits))
    }
}

impl FindDeadCode {
    /// Get context for a line.
    fn get_context(&self, source: &str, line: u32) -> String {
        source
            .lines()
            .nth(line as usize)
            .map(|l| {
                let trimmed = l.trim();
                if trimmed.len() > 60 {
                    format!("{}...", &trimmed[..57])
                } else {
                    trimmed.to_string()
                }
            })
            .unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_dead_code_validation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn unused() {}\nfn main() {}")
            .with_selection(0, 0, 1, 14);

        let op = FindDeadCode::new();
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_find_dead_code_empty_types() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {}")
            .with_selection(0, 0, 0, 12);

        let op = FindDeadCode::new()
            .only(Vec::new());

        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_find_unused_function() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn unused_func() {}\n\nfn main() {}")
            .with_selection(0, 0, 2, 14);

        let op = FindDeadCode::new().only(vec![DeadCodeType::UnusedFunctions]);
        let preview = op.preview(&ctx).unwrap();

        assert!(preview.diff.contains("unused_func"));
    }

    #[test]
    fn test_find_unused_variable() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let unused = 42;\n    println!(\"hello\");\n}")
            .with_selection(0, 0, 3, 1);

        let op = FindDeadCode::new().only(vec![DeadCodeType::UnusedVariables]);
        let preview = op.preview(&ctx).unwrap();

        assert!(preview.diff.contains("unused"));
    }

    #[test]
    fn test_skip_underscore_prefixed() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let _unused = 42;\n}")
            .with_selection(0, 0, 2, 1);

        let op = FindDeadCode::new().only(vec![DeadCodeType::UnusedVariables]);
        let preview = op.preview(&ctx).unwrap();

        // Should not report _unused
        assert!(!preview.diff.contains("_unused"));
    }

    #[test]
    fn test_dead_code_types() {
        let op = FindDeadCode::new()
            .include(DeadCodeType::EmptyBlocks)
            .include(DeadCodeType::CommentedCode)
            .exclude(DeadCodeType::UnusedFunctions);

        assert!(op.include_types.contains(&DeadCodeType::EmptyBlocks));
        assert!(op.include_types.contains(&DeadCodeType::CommentedCode));
        assert!(!op.include_types.contains(&DeadCodeType::UnusedFunctions));
    }
}
