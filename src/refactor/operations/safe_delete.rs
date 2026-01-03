//! Safe delete refactoring operations.

use std::path::PathBuf;

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::{RefactorError, Result};
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};
use super::RefactoringOperation;

/// Safely delete a symbol, checking for usages first.
#[derive(Debug, Clone)]
pub struct SafeDelete {
    /// Whether to force delete even if usages exist.
    pub force: bool,
    /// Whether to delete related items (like implementations).
    pub delete_related: bool,
    /// Search paths for finding usages.
    pub search_paths: Vec<PathBuf>,
}

impl Default for SafeDelete {
    fn default() -> Self {
        Self::new()
    }
}

impl SafeDelete {
    /// Create a new SafeDelete operation.
    pub fn new() -> Self {
        Self {
            force: false,
            delete_related: false,
            search_paths: Vec::new(),
        }
    }

    /// Force delete even if usages exist.
    pub fn force(mut self) -> Self {
        self.force = true;
        self
    }

    /// Also delete related items (implementations, etc.).
    pub fn with_related(mut self) -> Self {
        self.delete_related = true;
        self
    }

    /// Add a search path for finding usages.
    pub fn search_in(mut self, path: impl Into<PathBuf>) -> Self {
        self.search_paths.push(path.into());
        self
    }

    /// Find the symbol at the cursor.
    fn find_symbol(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<SymbolToDelete>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => {
                r#"
                (function_item name: (identifier) @name) @def
                (struct_item name: (type_identifier) @name) @def
                (enum_item name: (type_identifier) @name) @def
                (const_item name: (identifier) @name) @def
                (static_item name: (identifier) @name) @def
                (trait_item name: (type_identifier) @name) @def
                (type_item name: (type_identifier) @name) @def
                (let_declaration pattern: (identifier) @name) @def
                "#
            }
            "typescript" | "javascript" => {
                r#"
                (function_declaration name: (identifier) @name) @def
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                (variable_declarator name: (identifier) @name) @def
                (type_alias_declaration name: (type_identifier) @name) @def
                "#
            }
            "python" => {
                r#"
                (function_definition name: (identifier) @name) @def
                (class_definition name: (identifier) @name) @def
                (assignment left: (identifier) @name) @def
                "#
            }
            "go" => {
                r#"
                (function_declaration name: (identifier) @name) @def
                (method_declaration name: (field_identifier) @name) @def
                (type_declaration (type_spec name: (type_identifier) @name)) @def
                (var_declaration (var_spec name: (identifier) @name)) @def
                "#
            }
            "java" | "csharp" => {
                r#"
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                (method_declaration name: (identifier) @name) @def
                (field_declaration (variable_declarator name: (identifier) @name)) @def
                "#
            }
            "ruby" => {
                r#"
                (method name: (identifier) @name) @def
                (class name: (constant) @name) @def
                (module name: (constant) @name) @def
                "#
            }
            _ => return Ok(None),
        };

        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(None),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let cursor_line = ctx.target_range.start.line as usize;

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

            if let (Some(n), Some(range)) = (name, def_range) {
                // Check if cursor is on this definition
                if range.start.line as usize <= cursor_line
                    && cursor_line <= range.end.line as usize
                {
                    return Ok(Some(SymbolToDelete {
                        name: n.to_string(),
                        range,
                        kind: self.detect_kind(&ctx.source, &range),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Detect the kind of symbol.
    fn detect_kind(&self, source: &str, range: &Range) -> DeleteKind {
        let start_offset = self.range_to_offset(source, &range.start);
        let text = &source[start_offset..];
        let first_word = text.split_whitespace().next().unwrap_or("");

        match first_word {
            "fn" | "function" | "def" | "func" => DeleteKind::Function,
            "struct" | "class" => DeleteKind::Type,
            "enum" => DeleteKind::Enum,
            "trait" | "interface" => DeleteKind::Trait,
            "const" | "static" => DeleteKind::Constant,
            "let" | "var" => DeleteKind::Variable,
            "type" => DeleteKind::TypeAlias,
            "mod" | "module" => DeleteKind::Module,
            "impl" => DeleteKind::Implementation,
            _ => DeleteKind::Other,
        }
    }

    /// Convert position to offset.
    fn range_to_offset(&self, source: &str, pos: &Position) -> usize {
        let mut offset = 0;
        for (line_num, line) in source.lines().enumerate() {
            if line_num == pos.line as usize {
                return offset + (pos.character as usize).min(line.len());
            }
            offset += line.len() + 1;
        }
        offset.min(source.len())
    }

    /// Find usages of a symbol in the current file.
    fn find_usages_in_file(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
        symbol_name: &str,
        symbol_range: &Range,
    ) -> Result<Vec<UsageLocation>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query = lang.query("(identifier) @id")?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut usages = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if let Ok(text) = capture.node.utf8_text(source_bytes)
                    && text == symbol_name {
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
                        if range.start.line != symbol_range.start.line
                            || range.start.character != symbol_range.start.character
                        {
                            // Skip if this is within the definition range
                            let is_in_def = range.start.line >= symbol_range.start.line
                                && range.end.line <= symbol_range.end.line;

                            if !is_in_def {
                                usages.push(UsageLocation {
                                    file: ctx.target_file.clone(),
                                    range,
                                    context: self.get_line_context(&ctx.source, range.start.line),
                                });
                            }
                        }
                    }
            }
        }

        Ok(usages)
    }

    /// Get the line context for a usage.
    fn get_line_context(&self, source: &str, line: u32) -> String {
        source
            .lines()
            .nth(line as usize)
            .map(|l| l.trim().to_string())
            .unwrap_or_default()
    }

    /// Find related items (implementations, etc.).
    fn find_related_items(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
        symbol_name: &str,
    ) -> Result<Vec<Range>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let mut related = Vec::new();

        // For Rust, find impl blocks for the type
        if lang.name() == "rust" {
            let query_str = "(impl_item type: (type_identifier) @type) @impl";
            if let Ok(query) = lang.query(query_str) {
                let mut cursor = QueryCursor::new();
                let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

                while let Some(m) = matches.next() {
                    let mut type_name = None;
                    let mut impl_range = None;

                    for capture in m.captures {
                        let capture_name = query.capture_names()[capture.index as usize];
                        match capture_name {
                            "type" => {
                                type_name = capture.node.utf8_text(source_bytes).ok();
                            }
                            "impl" => {
                                let node = capture.node;
                                impl_range = Some(Range {
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

                    if let (Some(t), Some(r)) = (type_name, impl_range)
                        && t == symbol_name {
                            related.push(r);
                        }
                }
            }
        }

        Ok(related)
    }
}

/// Information about a symbol to delete.
#[derive(Debug, Clone)]
struct SymbolToDelete {
    name: String,
    range: Range,
    kind: DeleteKind,
}

/// Kind of item being deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeleteKind {
    Function,
    Type,
    Enum,
    Trait,
    Constant,
    Variable,
    TypeAlias,
    Module,
    Implementation,
    Other,
}

/// A usage location.
#[derive(Debug, Clone)]
pub struct UsageLocation {
    /// File containing the usage.
    pub file: PathBuf,
    /// Range of the usage.
    pub range: Range,
    /// Context (the line content).
    pub context: String,
}

impl RefactoringOperation for SafeDelete {
    fn name(&self) -> &'static str {
        "Safe Delete"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let symbol = self.find_symbol(ctx, lang)?;
        if symbol.is_none() {
            return Ok(ValidationResult::invalid(
                "No deletable symbol found at cursor position",
            ));
        }

        let symbol = symbol.unwrap();

        // Find usages
        let usages = self.find_usages_in_file(ctx, lang, &symbol.name, &symbol.range)?;

        if !usages.is_empty() && !self.force {
            let usage_list: Vec<String> = usages
                .iter()
                .take(5)
                .map(|u| format!("  - Line {}: {}", u.range.start.line + 1, u.context))
                .collect();

            let more = if usages.len() > 5 {
                format!("\n  ... and {} more", usages.len() - 5)
            } else {
                String::new()
            };

            return Ok(ValidationResult::invalid(format!(
                "Symbol '{}' has {} usage(s):\n{}{}",
                symbol.name,
                usages.len(),
                usage_list.join("\n"),
                more
            )));
        }

        if !usages.is_empty() && self.force {
            return Ok(ValidationResult::valid().with_warning(format!(
                "Force deleting '{}' with {} usage(s)",
                symbol.name,
                usages.len()
            )));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let symbol = self.find_symbol(ctx, lang)?.ok_or_else(|| {
            RefactorError::InvalidConfig("No deletable symbol found".to_string())
        })?;

        let mut preview =
            RefactoringPreview::new(format!("Delete {} '{}'", format!("{:?}", symbol.kind).to_lowercase(), symbol.name));

        // Delete the symbol
        let delete_range = Range {
            start: Position {
                line: symbol.range.start.line,
                character: 0,
            },
            end: Position {
                line: symbol.range.end.line + 1,
                character: 0,
            },
        };

        preview.add_edit(TextEdit::new(
            ctx.target_file.clone(),
            delete_range,
            String::new(),
        ));

        // Delete related items if requested
        let mut related_count = 0;
        if self.delete_related {
            let related = self.find_related_items(ctx, lang, &symbol.name)?;
            related_count = related.len();

            for range in related {
                let del_range = Range {
                    start: Position {
                        line: range.start.line,
                        character: 0,
                    },
                    end: Position {
                        line: range.end.line + 1,
                        character: 0,
                    },
                };

                preview.add_edit(TextEdit::new(
                    ctx.target_file.clone(),
                    del_range,
                    String::new(),
                ));
            }
        }

        let usages = self.find_usages_in_file(ctx, lang, &symbol.name, &symbol.range)?;

        let diff = format!(
            "Delete {} '{}' (lines {}-{})\nUsages: {}\nRelated items: {}{}",
            format!("{:?}", symbol.kind).to_lowercase(),
            symbol.name,
            symbol.range.start.line + 1,
            symbol.range.end.line + 1,
            usages.len(),
            related_count,
            if self.force && !usages.is_empty() {
                "\n⚠️ Force delete enabled"
            } else {
                ""
            }
        );
        preview = preview.with_diff(diff);

        Ok(preview)
    }

    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        let preview = self.preview(ctx)?;

        let mut new_source = ctx.source.clone();

        // Sort edits by position (descending) to apply from bottom to top
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
            "Deleted {} item(s)",
            edits.len()
        ))
        .with_file(ctx.target_file.clone())
        .with_edits(edits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_delete_no_usages() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn unused_function() { }\n\nfn main() { }")
            .with_selection(0, 0, 0, 24);

        let op = SafeDelete::new();
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_safe_delete_with_usages() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn used_func() { }\n\nfn main() { used_func(); }")
            .with_selection(0, 0, 0, 18);

        let op = SafeDelete::new();
        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
        assert!(result.errors[0].contains("usage"));
    }

    #[test]
    fn test_safe_delete_force() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn used_func() { }\n\nfn main() { used_func(); }")
            .with_selection(0, 0, 0, 18);

        let op = SafeDelete::new().force();
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_detect_kind() {
        let op = SafeDelete::new();

        let source = "fn test() {}";
        let range = Range {
            start: Position { line: 0, character: 0 },
            end: Position { line: 0, character: 12 },
        };
        assert_eq!(op.detect_kind(source, &range), DeleteKind::Function);

        let source = "struct Foo {}";
        assert_eq!(op.detect_kind(source, &range), DeleteKind::Type);

        let source = "const VALUE: i32 = 42;";
        assert_eq!(op.detect_kind(source, &range), DeleteKind::Constant);
    }

    #[test]
    fn test_find_usages() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn target() {}\nfn other() { target(); target(); }");

        let lang = ctx.language().unwrap();
        let op = SafeDelete::new();

        let symbol_range = Range {
            start: Position { line: 0, character: 3 },
            end: Position { line: 0, character: 9 },
        };

        let usages = op.find_usages_in_file(&ctx, lang, "target", &symbol_range).unwrap();
        assert_eq!(usages.len(), 2);
    }
}
