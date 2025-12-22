//! LSP-related type definitions.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A position in a text document (0-indexed line and character).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub line: u32,
    pub character: u32,
}

impl Position {
    /// Creates a new position.
    pub fn new(line: u32, character: u32) -> Self {
        Self { line, character }
    }

    /// Converts to lsp_types::Position.
    pub fn to_lsp(&self) -> lsp_types::Position {
        lsp_types::Position {
            line: self.line,
            character: self.character,
        }
    }

    /// Creates from lsp_types::Position.
    pub fn from_lsp(pos: lsp_types::Position) -> Self {
        Self {
            line: pos.line,
            character: pos.character,
        }
    }
}

/// A range in a text document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Range {
    pub start: Position,
    pub end: Position,
}

impl Range {
    /// Creates a new range.
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Converts to lsp_types::Range.
    pub fn to_lsp(&self) -> lsp_types::Range {
        lsp_types::Range {
            start: self.start.to_lsp(),
            end: self.end.to_lsp(),
        }
    }

    /// Creates from lsp_types::Range.
    pub fn from_lsp(range: lsp_types::Range) -> Self {
        Self {
            start: Position::from_lsp(range.start),
            end: Position::from_lsp(range.end),
        }
    }
}

/// A location in a document (file path + range).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location {
    pub path: PathBuf,
    pub range: Range,
}

impl Location {
    /// Creates a new location.
    pub fn new(path: impl Into<PathBuf>, range: Range) -> Self {
        Self {
            path: path.into(),
            range,
        }
    }
}

/// A text edit to apply to a document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextEdit {
    pub range: Range,
    pub new_text: String,
}

impl TextEdit {
    /// Creates a new text edit.
    pub fn new(range: Range, new_text: impl Into<String>) -> Self {
        Self {
            range,
            new_text: new_text.into(),
        }
    }

    /// Creates from lsp_types::TextEdit.
    pub fn from_lsp(edit: &lsp_types::TextEdit) -> Self {
        Self {
            range: Range::from_lsp(edit.range),
            new_text: edit.new_text.clone(),
        }
    }

    /// Converts to lsp_types::TextEdit.
    pub fn to_lsp(&self) -> lsp_types::TextEdit {
        lsp_types::TextEdit {
            range: self.range.to_lsp(),
            new_text: self.new_text.clone(),
        }
    }
}

/// A workspace edit containing changes to multiple files.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkspaceEdit {
    /// Map of file path to text edits.
    pub changes: HashMap<PathBuf, Vec<TextEdit>>,
}

impl WorkspaceEdit {
    /// Creates a new empty workspace edit.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds edits for a file.
    pub fn add_edits(&mut self, path: PathBuf, edits: Vec<TextEdit>) {
        self.changes.entry(path).or_default().extend(edits);
    }

    /// Returns the number of files affected.
    pub fn file_count(&self) -> usize {
        self.changes.len()
    }

    /// Returns the total number of edits.
    pub fn edit_count(&self) -> usize {
        self.changes.values().map(|v| v.len()).sum()
    }

    /// Checks if the edit is empty.
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }

    /// Applies the edits to the filesystem.
    pub fn apply(&self) -> crate::error::Result<()> {
        for (path, edits) in &self.changes {
            let content = std::fs::read_to_string(path)?;
            let new_content = apply_edits_to_string(&content, edits);
            std::fs::write(path, new_content)?;
        }
        Ok(())
    }

    /// Returns a preview of the changes without applying.
    pub fn preview(&self) -> crate::error::Result<HashMap<PathBuf, String>> {
        let mut result = HashMap::new();
        for (path, edits) in &self.changes {
            let content = std::fs::read_to_string(path)?;
            let new_content = apply_edits_to_string(&content, edits);
            result.insert(path.clone(), new_content);
        }
        Ok(result)
    }
}

/// Applies text edits to a string, returning the modified string.
fn apply_edits_to_string(content: &str, edits: &[TextEdit]) -> String {
    let lines: Vec<&str> = content.lines().collect();

    // Sort edits by position in reverse order (to apply from end to start)
    let mut sorted_edits: Vec<_> = edits.iter().collect();
    sorted_edits.sort_by(|a, b| {
        let a_pos = (a.range.start.line, a.range.start.character);
        let b_pos = (b.range.start.line, b.range.start.character);
        b_pos.cmp(&a_pos) // Reverse order
    });

    // Convert to byte offsets and apply
    let mut result = content.to_string();

    for edit in sorted_edits {
        let start_offset = position_to_offset(&lines, edit.range.start);
        let end_offset = position_to_offset(&lines, edit.range.end);

        if let (Some(start), Some(end)) = (start_offset, end_offset) {
            result.replace_range(start..end, &edit.new_text);
        }
    }

    result
}

/// Converts a Position to a byte offset in the content.
fn position_to_offset(lines: &[&str], pos: Position) -> Option<usize> {
    let line_idx = pos.line as usize;
    if line_idx > lines.len() {
        return None;
    }

    let mut offset = 0;
    for (i, line) in lines.iter().enumerate() {
        if i == line_idx {
            let char_offset = pos.character as usize;
            // Handle UTF-8 properly
            let byte_offset: usize = line
                .char_indices()
                .take(char_offset)
                .last()
                .map(|(i, c)| i + c.len_utf8())
                .unwrap_or(0);
            return Some(offset + byte_offset.min(line.len()));
        }
        offset += line.len() + 1; // +1 for newline
    }

    // Position at end of file
    if line_idx == lines.len() {
        Some(offset.saturating_sub(1))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_to_offset() {
        let content = "line1\nline2\nline3";
        let lines: Vec<&str> = content.lines().collect();

        assert_eq!(position_to_offset(&lines, Position::new(0, 0)), Some(0));
        assert_eq!(position_to_offset(&lines, Position::new(0, 3)), Some(3));
        assert_eq!(position_to_offset(&lines, Position::new(1, 0)), Some(6));
        assert_eq!(position_to_offset(&lines, Position::new(2, 0)), Some(12));
    }

    #[test]
    fn test_apply_edits() {
        let content = "fn old_name() {}";
        let edits = vec![TextEdit::new(
            Range::new(Position::new(0, 3), Position::new(0, 11)),
            "new_name",
        )];

        let result = apply_edits_to_string(content, &edits);
        assert_eq!(result, "fn new_name() {}");
    }

    #[test]
    fn test_apply_multiple_edits() {
        let content = "let x = old;\nlet y = old;";
        let edits = vec![
            TextEdit::new(
                Range::new(Position::new(0, 8), Position::new(0, 11)),
                "new",
            ),
            TextEdit::new(
                Range::new(Position::new(1, 8), Position::new(1, 11)),
                "new",
            ),
        ];

        let result = apply_edits_to_string(content, &edits);
        assert_eq!(result, "let x = new;\nlet y = new;");
    }

    #[test]
    fn test_workspace_edit_counts() {
        let mut edit = WorkspaceEdit::new();
        edit.add_edits(
            PathBuf::from("file1.rs"),
            vec![TextEdit::new(
                Range::new(Position::new(0, 0), Position::new(0, 3)),
                "foo",
            )],
        );
        edit.add_edits(
            PathBuf::from("file2.rs"),
            vec![
                TextEdit::new(
                    Range::new(Position::new(0, 0), Position::new(0, 3)),
                    "bar",
                ),
                TextEdit::new(
                    Range::new(Position::new(1, 0), Position::new(1, 3)),
                    "baz",
                ),
            ],
        );

        assert_eq!(edit.file_count(), 2);
        assert_eq!(edit.edit_count(), 3);
    }
}
