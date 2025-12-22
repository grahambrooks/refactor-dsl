//! Diff generation for previewing changes.

use similar::{ChangeTag, TextDiff};
use std::fmt::Write;
use std::path::Path;

/// Generates a unified diff between two strings.
pub fn unified_diff(original: &str, modified: &str, path: &Path) -> String {
    let diff = TextDiff::from_lines(original, modified);
    let mut output = String::new();

    writeln!(&mut output, "--- a/{}", path.display()).unwrap();
    writeln!(&mut output, "+++ b/{}", path.display()).unwrap();

    for (idx, group) in diff.grouped_ops(3).iter().enumerate() {
        if idx > 0 {
            writeln!(&mut output).unwrap();
        }

        for op in group {
            for change in diff.iter_changes(op) {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };

                write!(&mut output, "{}{}", sign, change.value()).unwrap();
            }
        }
    }

    output
}

/// Represents a summary of changes.
#[derive(Debug, Default)]
pub struct DiffSummary {
    pub files_changed: usize,
    pub insertions: usize,
    pub deletions: usize,
}

impl DiffSummary {
    /// Creates a summary from original and modified content.
    pub fn from_diff(original: &str, modified: &str) -> Self {
        let diff = TextDiff::from_lines(original, modified);
        let mut insertions = 0;
        let mut deletions = 0;

        for change in diff.iter_all_changes() {
            match change.tag() {
                ChangeTag::Insert => insertions += 1,
                ChangeTag::Delete => deletions += 1,
                ChangeTag::Equal => {}
            }
        }

        Self {
            files_changed: if insertions > 0 || deletions > 0 { 1 } else { 0 },
            insertions,
            deletions,
        }
    }

    /// Combines two summaries.
    pub fn merge(&mut self, other: &DiffSummary) {
        self.files_changed += other.files_changed;
        self.insertions += other.insertions;
        self.deletions += other.deletions;
    }
}

impl std::fmt::Display for DiffSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} file(s) changed, {} insertions(+), {} deletions(-)",
            self.files_changed, self.insertions, self.deletions
        )
    }
}

/// Colorized diff output for terminal display.
pub fn colorized_diff(original: &str, modified: &str, path: &Path) -> String {
    let diff = TextDiff::from_lines(original, modified);
    let mut output = String::new();

    // ANSI color codes
    const RED: &str = "\x1b[31m";
    const GREEN: &str = "\x1b[32m";
    const CYAN: &str = "\x1b[36m";
    const RESET: &str = "\x1b[0m";

    writeln!(&mut output, "{}--- a/{}{}", CYAN, path.display(), RESET).unwrap();
    writeln!(&mut output, "{}+++ b/{}{}", CYAN, path.display(), RESET).unwrap();

    for group in diff.grouped_ops(3).iter() {
        for op in group {
            for change in diff.iter_changes(op) {
                let (sign, color) = match change.tag() {
                    ChangeTag::Delete => ("-", RED),
                    ChangeTag::Insert => ("+", GREEN),
                    ChangeTag::Equal => (" ", ""),
                };

                if color.is_empty() {
                    write!(&mut output, "{}{}", sign, change.value()).unwrap();
                } else {
                    write!(&mut output, "{}{}{}{}", color, sign, change.value(), RESET).unwrap();
                }
            }
        }
    }

    output
}
