//! Refactoring context for operations.

use std::path::PathBuf;

use crate::error::{RefactorError, Result};
use crate::lang::{Language, LanguageRegistry};
use crate::lsp::{Position, Range};
use crate::scope::ScopeAnalyzer;

/// Context for a refactoring operation.
pub struct RefactoringContext {
    /// Root of the workspace.
    pub workspace_root: PathBuf,
    /// Target file for the operation.
    pub target_file: PathBuf,
    /// Target range in the file (selection).
    pub target_range: Range,
    /// The source code content.
    pub source: String,
    /// Language registry.
    pub registry: LanguageRegistry,
    /// Scope analyzer (lazy initialized).
    scope_analyzer: Option<ScopeAnalyzer>,
}

impl RefactoringContext {
    /// Create a new refactoring context.
    pub fn new(workspace_root: impl Into<PathBuf>, target_file: impl Into<PathBuf>) -> Self {
        Self {
            workspace_root: workspace_root.into(),
            target_file: target_file.into(),
            target_range: Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 0,
                },
            },
            source: String::new(),
            registry: LanguageRegistry::new(),
            scope_analyzer: None,
        }
    }

    /// Set the target range.
    pub fn with_range(mut self, range: Range) -> Self {
        self.target_range = range;
        self
    }

    /// Set the target range from line/column positions.
    pub fn with_selection(
        mut self,
        start_line: u32,
        start_col: u32,
        end_line: u32,
        end_col: u32,
    ) -> Self {
        self.target_range = Range {
            start: Position {
                line: start_line,
                character: start_col,
            },
            end: Position {
                line: end_line,
                character: end_col,
            },
        };
        self
    }

    /// Set the source code.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = source.into();
        self
    }

    /// Load source from the target file.
    pub fn load_source(mut self) -> Result<Self> {
        self.source = std::fs::read_to_string(&self.target_file)?;
        Ok(self)
    }

    /// Get the language for the target file.
    pub fn language(&self) -> Option<&dyn Language> {
        self.registry.detect(&self.target_file)
    }

    /// Get the selected text.
    pub fn selected_text(&self) -> &str {
        let start_offset = self.position_to_offset(&self.target_range.start);
        let end_offset = self.position_to_offset(&self.target_range.end);

        if start_offset <= end_offset && end_offset <= self.source.len() {
            &self.source[start_offset..end_offset]
        } else {
            ""
        }
    }

    /// Convert a position to a byte offset.
    pub fn position_to_offset(&self, pos: &Position) -> usize {
        let mut offset = 0;
        for (line_num, line) in self.source.lines().enumerate() {
            if line_num == pos.line as usize {
                return offset + (pos.character as usize).min(line.len());
            }
            offset += line.len() + 1; // +1 for newline
        }
        offset.min(self.source.len())
    }

    /// Convert a byte offset to a position.
    pub fn offset_to_position(&self, offset: usize) -> Position {
        let mut line = 0;
        let mut col = 0;
        let mut current_offset = 0;

        for ch in self.source.chars() {
            if current_offset >= offset {
                break;
            }
            if ch == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
            current_offset += ch.len_utf8();
        }

        Position {
            line,
            character: col,
        }
    }

    /// Get or create the scope analyzer.
    pub fn scope_analyzer(&mut self) -> Result<&mut ScopeAnalyzer> {
        if self.scope_analyzer.is_none() {
            let mut analyzer = ScopeAnalyzer::new();
            analyzer.analyze_file(&self.target_file, &self.source)?;
            self.scope_analyzer = Some(analyzer);
        }
        Ok(self.scope_analyzer.as_mut().unwrap())
    }

    /// Get the line at a given line number.
    pub fn get_line(&self, line_num: u32) -> Option<&str> {
        self.source.lines().nth(line_num as usize)
    }

    /// Get the indentation of a line.
    pub fn get_indentation(&self, line_num: u32) -> String {
        self.get_line(line_num)
            .map(|line| {
                let indent_len = line.len() - line.trim_start().len();
                line[..indent_len].to_string()
            })
            .unwrap_or_default()
    }

    /// Validate the context before an operation.
    pub fn validate(&self) -> Result<()> {
        if self.source.is_empty() {
            return Err(RefactorError::InvalidConfig(
                "Source code is empty".to_string(),
            ));
        }

        if self.language().is_none() {
            return Err(RefactorError::UnsupportedLanguage(
                self.target_file
                    .extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            ));
        }

        Ok(())
    }
}

/// Result of validating a refactoring operation.
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether the operation is valid.
    pub is_valid: bool,
    /// Error messages if invalid.
    pub errors: Vec<String>,
    /// Warning messages.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a valid result.
    pub fn valid() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Create an invalid result with an error.
    pub fn invalid(error: impl Into<String>) -> Self {
        Self {
            is_valid: false,
            errors: vec![error.into()],
            warnings: Vec::new(),
        }
    }

    /// Add a warning.
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }

    /// Add an error.
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.is_valid = false;
        self.errors.push(error.into());
        self
    }
}

/// Preview of a refactoring operation.
#[derive(Debug, Clone)]
pub struct RefactoringPreview {
    /// Description of the operation.
    pub description: String,
    /// Files that will be modified.
    pub affected_files: Vec<PathBuf>,
    /// Text edits to apply.
    pub edits: Vec<TextEdit>,
    /// Diff preview.
    pub diff: String,
}

impl RefactoringPreview {
    /// Create a new preview.
    pub fn new(description: impl Into<String>) -> Self {
        Self {
            description: description.into(),
            affected_files: Vec::new(),
            edits: Vec::new(),
            diff: String::new(),
        }
    }

    /// Add an affected file.
    pub fn add_file(&mut self, file: PathBuf) {
        if !self.affected_files.contains(&file) {
            self.affected_files.push(file);
        }
    }

    /// Add a text edit.
    pub fn add_edit(&mut self, edit: TextEdit) {
        self.add_file(edit.file.clone());
        self.edits.push(edit);
    }

    /// Set the diff.
    pub fn with_diff(mut self, diff: impl Into<String>) -> Self {
        self.diff = diff.into();
        self
    }
}

/// A text edit to apply.
#[derive(Debug, Clone)]
pub struct TextEdit {
    /// File to edit.
    pub file: PathBuf,
    /// Range to replace.
    pub range: Range,
    /// New text.
    pub new_text: String,
}

impl TextEdit {
    /// Create a new text edit.
    pub fn new(file: impl Into<PathBuf>, range: Range, new_text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            range,
            new_text: new_text.into(),
        }
    }

    /// Create an insertion edit.
    pub fn insert(file: impl Into<PathBuf>, position: Position, text: impl Into<String>) -> Self {
        Self {
            file: file.into(),
            range: Range {
                start: position,
                end: position,
            },
            new_text: text.into(),
        }
    }

    /// Create a deletion edit.
    pub fn delete(file: impl Into<PathBuf>, range: Range) -> Self {
        Self {
            file: file.into(),
            range,
            new_text: String::new(),
        }
    }
}

/// Result of applying a refactoring operation.
#[derive(Debug, Clone)]
pub struct RefactoringResult {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Description of what was done.
    pub description: String,
    /// Files that were modified.
    pub modified_files: Vec<PathBuf>,
    /// Edits that were applied.
    pub applied_edits: Vec<TextEdit>,
    /// Error message if failed.
    pub error: Option<String>,
}

impl RefactoringResult {
    /// Create a successful result.
    pub fn success(description: impl Into<String>) -> Self {
        Self {
            success: true,
            description: description.into(),
            modified_files: Vec::new(),
            applied_edits: Vec::new(),
            error: None,
        }
    }

    /// Create a failed result.
    pub fn failure(error: impl Into<String>) -> Self {
        Self {
            success: false,
            description: String::new(),
            modified_files: Vec::new(),
            applied_edits: Vec::new(),
            error: Some(error.into()),
        }
    }

    /// Add a modified file.
    pub fn with_file(mut self, file: PathBuf) -> Self {
        if !self.modified_files.contains(&file) {
            self.modified_files.push(file);
        }
        self
    }

    /// Add applied edits.
    pub fn with_edits(mut self, edits: Vec<TextEdit>) -> Self {
        for edit in &edits {
            if !self.modified_files.contains(&edit.file) {
                self.modified_files.push(edit.file.clone());
            }
        }
        self.applied_edits = edits;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let ctx = RefactoringContext::new("/workspace", "src/main.rs")
            .with_source("fn main() {}\n")
            .with_selection(0, 0, 0, 12);

        assert_eq!(ctx.target_file, PathBuf::from("src/main.rs"));
        assert_eq!(ctx.selected_text(), "fn main() {}");
    }

    #[test]
    fn test_position_to_offset() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("line one\nline two\nline three");

        assert_eq!(
            ctx.position_to_offset(&Position {
                line: 0,
                character: 0
            }),
            0
        );
        assert_eq!(
            ctx.position_to_offset(&Position {
                line: 1,
                character: 0
            }),
            9
        );
        assert_eq!(
            ctx.position_to_offset(&Position {
                line: 1,
                character: 4
            }),
            13
        );
    }

    #[test]
    fn test_offset_to_position() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("line one\nline two\nline three");

        let pos = ctx.offset_to_position(13);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.character, 4);
    }

    #[test]
    fn test_get_indentation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let x = 1;\n}");

        assert_eq!(ctx.get_indentation(0), "");
        assert_eq!(ctx.get_indentation(1), "    ");
        assert_eq!(ctx.get_indentation(2), "");
    }

    #[test]
    fn test_validation_result() {
        let valid = ValidationResult::valid();
        assert!(valid.is_valid);

        let invalid = ValidationResult::invalid("Something wrong");
        assert!(!invalid.is_valid);
        assert_eq!(invalid.errors.len(), 1);

        let with_warning = ValidationResult::valid().with_warning("Be careful");
        assert!(with_warning.is_valid);
        assert_eq!(with_warning.warnings.len(), 1);
    }

    #[test]
    fn test_text_edit() {
        let edit = TextEdit::new(
            "test.rs",
            Range {
                start: Position {
                    line: 0,
                    character: 0,
                },
                end: Position {
                    line: 0,
                    character: 5,
                },
            },
            "hello",
        );

        assert_eq!(edit.file, PathBuf::from("test.rs"));
        assert_eq!(edit.new_text, "hello");
    }
}
