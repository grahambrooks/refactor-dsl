//! Move refactoring operations.

use std::path::{Path, PathBuf};

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::{RefactorError, Result};
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::RefactoringOperation;
use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};

/// Move a symbol to a different file.
#[derive(Debug, Clone)]
pub struct MoveToFile {
    /// Target file to move the symbol to.
    pub target_file: PathBuf,
    /// Whether to update imports in other files.
    pub update_imports: bool,
    /// Whether to add re-export from original location.
    pub add_reexport: bool,
}

impl MoveToFile {
    /// Create a new MoveToFile operation.
    pub fn new(target_file: impl Into<PathBuf>) -> Self {
        Self {
            target_file: target_file.into(),
            update_imports: true,
            add_reexport: false,
        }
    }

    /// Don't update imports in other files.
    pub fn skip_import_updates(mut self) -> Self {
        self.update_imports = false;
        self
    }

    /// Add a re-export from the original location for backwards compatibility.
    pub fn with_reexport(mut self) -> Self {
        self.add_reexport = true;
        self
    }

    /// Find the symbol definition at the cursor.
    fn find_symbol(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<SymbolInfo>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => {
                r#"
                (function_item name: (identifier) @name) @def
                (struct_item name: (type_identifier) @name) @def
                (enum_item name: (type_identifier) @name) @def
                (impl_item type: (type_identifier) @name) @def
                (trait_item name: (type_identifier) @name) @def
                (const_item name: (identifier) @name) @def
                (static_item name: (identifier) @name) @def
                (type_item name: (type_identifier) @name) @def
                "#
            }
            "typescript" | "javascript" => {
                r#"
                (function_declaration name: (identifier) @name) @def
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                (type_alias_declaration name: (type_identifier) @name) @def
                (enum_declaration name: (identifier) @name) @def
                "#
            }
            "python" => {
                r#"
                (function_definition name: (identifier) @name) @def
                (class_definition name: (identifier) @name) @def
                "#
            }
            "go" => {
                r#"
                (function_declaration name: (identifier) @name) @def
                (method_declaration name: (field_identifier) @name) @def
                (type_declaration (type_spec name: (type_identifier) @name)) @def
                "#
            }
            "java" | "csharp" => {
                r#"
                (class_declaration name: (identifier) @name) @def
                (interface_declaration name: (identifier) @name) @def
                (method_declaration name: (identifier) @name) @def
                (enum_declaration name: (identifier) @name) @def
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
            let mut full_text = None;

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
                        full_text = node.utf8_text(source_bytes).ok();
                    }
                    _ => {}
                }
            }

            if let (Some(n), Some(range), Some(text)) = (name, def_range, full_text) {
                // Check if cursor is within this definition
                if range.start.line as usize <= cursor_line
                    && cursor_line <= range.end.line as usize
                {
                    return Ok(Some(SymbolInfo {
                        name: n.to_string(),
                        full_text: text.to_string(),
                        range,
                        kind: self.detect_symbol_kind(text, lang.name()),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Detect the kind of symbol from its text.
    fn detect_symbol_kind(&self, text: &str, lang_name: &str) -> SymbolKind {
        let text = text.trim();
        match lang_name {
            "rust" => {
                if text.starts_with("fn ") {
                    SymbolKind::Function
                } else if text.starts_with("struct ") {
                    SymbolKind::Struct
                } else if text.starts_with("enum ") {
                    SymbolKind::Enum
                } else if text.starts_with("trait ") {
                    SymbolKind::Trait
                } else if text.starts_with("impl ") {
                    SymbolKind::Impl
                } else if text.starts_with("const ") {
                    SymbolKind::Constant
                } else if text.starts_with("type ") {
                    SymbolKind::TypeAlias
                } else {
                    SymbolKind::Other
                }
            }
            "typescript" | "javascript" => {
                if text.starts_with("function ") {
                    SymbolKind::Function
                } else if text.starts_with("class ") {
                    SymbolKind::Class
                } else if text.starts_with("interface ") {
                    SymbolKind::Interface
                } else if text.starts_with("type ") {
                    SymbolKind::TypeAlias
                } else if text.starts_with("enum ") {
                    SymbolKind::Enum
                } else {
                    SymbolKind::Other
                }
            }
            "python" => {
                if text.starts_with("def ") {
                    SymbolKind::Function
                } else if text.starts_with("class ") {
                    SymbolKind::Class
                } else {
                    SymbolKind::Other
                }
            }
            _ => SymbolKind::Other,
        }
    }

    /// Generate import statement for the target file.
    #[allow(dead_code)]
    fn generate_import(&self, symbol_name: &str, from_file: &Path, lang_name: &str) -> String {
        let module_path = self.file_to_module_path(from_file, lang_name);

        match lang_name {
            "rust" => format!("use {}::{};\n", module_path, symbol_name),
            "typescript" | "javascript" => {
                format!("import {{ {} }} from '{}';\n", symbol_name, module_path)
            }
            "python" => format!("from {} import {}\n", module_path, symbol_name),
            "go" => format!("// import from {}\n", module_path),
            "java" => format!("import {};\n", module_path),
            "csharp" => format!("using {};\n", module_path),
            "ruby" => format!("require_relative '{}'\n", module_path),
            _ => String::new(),
        }
    }

    /// Generate re-export statement.
    fn generate_reexport(&self, symbol_name: &str, target_file: &Path, lang_name: &str) -> String {
        let module_path = self.file_to_module_path(target_file, lang_name);

        match lang_name {
            "rust" => format!("pub use {}::{};\n", module_path, symbol_name),
            "typescript" | "javascript" => {
                format!("export {{ {} }} from '{}';\n", symbol_name, module_path)
            }
            "python" => format!("from {} import {}\n", module_path, symbol_name),
            _ => String::new(),
        }
    }

    /// Convert file path to module path.
    fn file_to_module_path(&self, file: &Path, lang_name: &str) -> String {
        let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");

        match lang_name {
            "rust" => {
                // Convert path/to/file.rs -> path::to::file
                file.with_extension("")
                    .to_string_lossy()
                    .replace(['/', '\\'], "::")
            }
            "typescript" | "javascript" => {
                // Convert to relative path
                format!("./{}", stem)
            }
            "python" => {
                // Convert path/to/file.py -> path.to.file
                file.with_extension("")
                    .to_string_lossy()
                    .replace(['/', '\\'], ".")
            }
            _ => stem.to_string(),
        }
    }
}

/// Information about a symbol to move.
#[derive(Debug, Clone)]
struct SymbolInfo {
    name: String,
    full_text: String,
    range: Range,
    kind: SymbolKind,
}

/// Kind of symbol.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Function,
    Struct,
    Class,
    Interface,
    Enum,
    Trait,
    Impl,
    Constant,
    TypeAlias,
    Module,
    Other,
}

impl RefactoringOperation for MoveToFile {
    fn name(&self) -> &'static str {
        "Move to File"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let symbol = self.find_symbol(ctx, lang)?;
        if symbol.is_none() {
            return Ok(ValidationResult::invalid(
                "No movable symbol found at cursor position",
            ));
        }

        // Check that target file is different from source
        if ctx.target_file == self.target_file {
            return Ok(ValidationResult::invalid(
                "Target file is the same as source file",
            ));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let symbol = self
            .find_symbol(ctx, lang)?
            .ok_or_else(|| RefactorError::InvalidConfig("No movable symbol found".to_string()))?;

        let mut preview = RefactoringPreview::new(format!(
            "Move '{}' to '{}'",
            symbol.name,
            self.target_file.display()
        ));

        // Delete from source file
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

        // Add re-export if requested
        if self.add_reexport {
            let reexport = self.generate_reexport(&symbol.name, &self.target_file, lang.name());
            preview.add_edit(TextEdit::insert(
                ctx.target_file.clone(),
                Position {
                    line: symbol.range.start.line,
                    character: 0,
                },
                reexport,
            ));
        }

        // Add to target file (at the end)
        preview.add_edit(TextEdit::insert(
            self.target_file.clone(),
            Position {
                line: u32::MAX, // Append at end
                character: 0,
            },
            format!("\n{}\n", symbol.full_text),
        ));

        let diff = format!(
            "Move {} '{}' from {} to {}\n- Delete from source\n+ Add to target{}",
            format!("{:?}", symbol.kind).to_lowercase(),
            symbol.name,
            ctx.target_file.display(),
            self.target_file.display(),
            if self.add_reexport {
                "\n+ Add re-export"
            } else {
                ""
            }
        );
        preview = preview.with_diff(diff);

        Ok(preview)
    }

    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let symbol = self
            .find_symbol(ctx, lang)?
            .ok_or_else(|| RefactorError::InvalidConfig("No movable symbol found".to_string()))?;

        // Read target file content (or create empty)
        let target_content = std::fs::read_to_string(&self.target_file).unwrap_or_default();

        // Remove from source
        let start_offset = ctx.position_to_offset(&symbol.range.start);
        let end_offset = ctx.position_to_offset(&symbol.range.end);

        // Extend to full lines
        let line_start = ctx.source[..start_offset]
            .rfind('\n')
            .map(|i| i + 1)
            .unwrap_or(0);
        let line_end = ctx.source[end_offset..]
            .find('\n')
            .map(|i| end_offset + i + 1)
            .unwrap_or(ctx.source.len());

        let mut new_source = ctx.source.clone();
        let removed_text = new_source[line_start..line_end].to_string();
        new_source.replace_range(line_start..line_end, "");

        // Add re-export if requested
        if self.add_reexport {
            let reexport = self.generate_reexport(&symbol.name, &self.target_file, lang.name());
            new_source.insert_str(line_start, &reexport);
        }

        // Add to target
        let new_target = format!("{}\n{}", target_content.trim_end(), removed_text.trim());

        // Write files
        std::fs::write(&ctx.target_file, &new_source)?;
        std::fs::write(&self.target_file, &new_target)?;
        ctx.source = new_source;

        Ok(RefactoringResult::success(format!(
            "Moved '{}' to '{}'",
            symbol.name,
            self.target_file.display()
        ))
        .with_file(ctx.target_file.clone())
        .with_file(self.target_file.clone()))
    }
}

/// Move a symbol between modules (within the same file or across files).
#[derive(Debug, Clone)]
pub struct MoveBetweenModules {
    /// Target module path (e.g., "crate::utils" or "super::helpers").
    pub target_module: String,
    /// Whether to update all references.
    pub update_references: bool,
}

impl MoveBetweenModules {
    /// Create a new MoveBetweenModules operation.
    pub fn new(target_module: impl Into<String>) -> Self {
        Self {
            target_module: target_module.into(),
            update_references: true,
        }
    }

    /// Skip updating references.
    pub fn skip_reference_updates(mut self) -> Self {
        self.update_references = false;
        self
    }

    /// Find the current module of the symbol.
    fn find_current_module(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<String>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        // For Rust, find mod declarations
        if lang.name() == "rust" {
            let query_str = "(mod_item name: (identifier) @name) @mod";
            let query = match lang.query(query_str) {
                Ok(q) => q,
                Err(_) => return Ok(None),
            };

            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

            let cursor_line = ctx.target_range.start.line as usize;
            let mut containing_mod = None;

            while let Some(m) = matches.next() {
                let mut name = None;
                let mut mod_start = 0;
                let mut mod_end = 0;

                for capture in m.captures {
                    let capture_name = query.capture_names()[capture.index as usize];
                    match capture_name {
                        "name" => {
                            name = capture.node.utf8_text(source_bytes).ok();
                        }
                        "mod" => {
                            mod_start = capture.node.start_position().row;
                            mod_end = capture.node.end_position().row;
                        }
                        _ => {}
                    }
                }

                if let Some(n) = name
                    && mod_start <= cursor_line
                    && cursor_line <= mod_end
                {
                    containing_mod = Some(n.to_string());
                }
            }

            return Ok(containing_mod);
        }

        Ok(None)
    }
}

impl RefactoringOperation for MoveBetweenModules {
    fn name(&self) -> &'static str {
        "Move Between Modules"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        // Only supported for Rust currently
        if lang.name() != "rust" {
            return Ok(ValidationResult::invalid(
                "Move between modules is currently only supported for Rust",
            )
            .with_warning("Use MoveToFile for other languages"));
        }

        if self.target_module.is_empty() {
            return Ok(ValidationResult::invalid("Target module cannot be empty"));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let current_module = self.find_current_module(ctx, lang)?;
        let selected = ctx.selected_text();

        let mut preview =
            RefactoringPreview::new(format!("Move selection to module '{}'", self.target_module));

        let diff = format!(
            "Move from {} to {}\nSelected: {}",
            current_module.as_deref().unwrap_or("root"),
            self.target_module,
            if selected.len() > 50 {
                format!("{}...", &selected[..50])
            } else {
                selected.to_string()
            }
        );
        preview = preview.with_diff(diff);

        // This is a complex operation that would require:
        // 1. Finding or creating the target module
        // 2. Moving the code
        // 3. Updating all references
        // For now, provide a preview of what would happen

        Ok(preview)
    }

    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        // This is a complex operation - for now return a message
        // indicating what manual steps are needed
        let selected = ctx.selected_text();

        Ok(RefactoringResult::success(format!(
            "To complete this refactoring:\n\
             1. Create or locate module '{}'\n\
             2. Move the selected code there\n\
             3. Update imports/use statements\n\
             Selected code:\n{}",
            self.target_module,
            if selected.len() > 200 {
                format!("{}...", &selected[..200])
            } else {
                selected.to_string()
            }
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_to_file_validation() {
        let ctx = RefactoringContext::new("/workspace", "src/lib.rs")
            .with_source("fn hello() { println!(\"hello\"); }")
            .with_selection(0, 0, 0, 34);

        let op = MoveToFile::new("src/utils.rs");
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_move_to_file_same_file() {
        let ctx = RefactoringContext::new("/workspace", "src/lib.rs")
            .with_source("fn hello() {}")
            .with_selection(0, 0, 0, 13);

        let op = MoveToFile::new("src/lib.rs");
        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_move_between_modules_rust_only() {
        let ctx = RefactoringContext::new("/workspace", "src/lib.ts")
            .with_source("function hello() {}")
            .with_selection(0, 0, 0, 19);

        let op = MoveBetweenModules::new("utils");
        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_symbol_kind_detection() {
        let op = MoveToFile::new("target.rs");

        assert_eq!(
            op.detect_symbol_kind("fn foo() {}", "rust"),
            SymbolKind::Function
        );
        assert_eq!(
            op.detect_symbol_kind("struct Bar {}", "rust"),
            SymbolKind::Struct
        );
        assert_eq!(
            op.detect_symbol_kind("enum Baz {}", "rust"),
            SymbolKind::Enum
        );
        assert_eq!(
            op.detect_symbol_kind("class Foo {}", "typescript"),
            SymbolKind::Class
        );
        assert_eq!(
            op.detect_symbol_kind("def foo():", "python"),
            SymbolKind::Function
        );
    }

    #[test]
    fn test_file_to_module_path() {
        let op = MoveToFile::new("target.rs");

        assert_eq!(
            op.file_to_module_path(&PathBuf::from("src/utils/helpers.rs"), "rust"),
            "src::utils::helpers"
        );
        assert_eq!(
            op.file_to_module_path(&PathBuf::from("utils.ts"), "typescript"),
            "./utils"
        );
        assert_eq!(
            op.file_to_module_path(&PathBuf::from("utils/helpers.py"), "python"),
            "utils.helpers"
        );
    }
}
