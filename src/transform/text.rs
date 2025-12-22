//! Text-based transformations using regex patterns.

use super::Transform;
use crate::error::Result;
use regex::Regex;
use std::path::Path;

/// Text-based transformation using regex replacement.
pub struct TextTransform {
    kind: TextTransformKind,
}

enum TextTransformKind {
    Replace { pattern: Regex, replacement: String },
    ReplaceLiteral { needle: String, replacement: String },
    PrependLine { pattern: Regex, prefix: String },
    AppendLine { pattern: Regex, suffix: String },
    DeleteLines { pattern: Regex },
    InsertAfter { pattern: Regex, content: String },
    InsertBefore { pattern: Regex, content: String },
}

impl TextTransform {
    /// Creates a regex replacement transform.
    pub fn replace(pattern: &str, replacement: &str) -> Self {
        Self {
            kind: TextTransformKind::Replace {
                pattern: Regex::new(pattern).expect("invalid regex"),
                replacement: replacement.to_string(),
            },
        }
    }

    /// Creates a replacement transform from a pre-compiled regex.
    pub fn replace_regex(pattern: Regex, replacement: impl Into<String>) -> Self {
        Self {
            kind: TextTransformKind::Replace {
                pattern,
                replacement: replacement.into(),
            },
        }
    }

    /// Creates a literal string replacement transform.
    pub fn replace_literal(needle: &str, replacement: &str) -> Self {
        Self {
            kind: TextTransformKind::ReplaceLiteral {
                needle: needle.to_string(),
                replacement: replacement.to_string(),
            },
        }
    }

    /// Creates a transform that prepends text to matching lines.
    pub fn prepend_line(pattern: &str, prefix: &str) -> Result<Self> {
        Ok(Self {
            kind: TextTransformKind::PrependLine {
                pattern: Regex::new(pattern)?,
                prefix: prefix.to_string(),
            },
        })
    }

    /// Creates a transform that appends text to matching lines.
    pub fn append_line(pattern: &str, suffix: &str) -> Result<Self> {
        Ok(Self {
            kind: TextTransformKind::AppendLine {
                pattern: Regex::new(pattern)?,
                suffix: suffix.to_string(),
            },
        })
    }

    /// Creates a transform that deletes matching lines.
    pub fn delete_lines(pattern: &str) -> Result<Self> {
        Ok(Self {
            kind: TextTransformKind::DeleteLines {
                pattern: Regex::new(pattern)?,
            },
        })
    }

    /// Creates a transform that inserts content after matching lines.
    pub fn insert_after(pattern: &str, content: &str) -> Result<Self> {
        Ok(Self {
            kind: TextTransformKind::InsertAfter {
                pattern: Regex::new(pattern)?,
                content: content.to_string(),
            },
        })
    }

    /// Creates a transform that inserts content before matching lines.
    pub fn insert_before(pattern: &str, content: &str) -> Result<Self> {
        Ok(Self {
            kind: TextTransformKind::InsertBefore {
                pattern: Regex::new(pattern)?,
                content: content.to_string(),
            },
        })
    }
}

impl Transform for TextTransform {
    fn apply(&self, source: &str, _path: &Path) -> Result<String> {
        match &self.kind {
            TextTransformKind::Replace { pattern, replacement } => {
                Ok(pattern.replace_all(source, replacement.as_str()).into_owned())
            }
            TextTransformKind::ReplaceLiteral { needle, replacement } => {
                Ok(source.replace(needle, replacement))
            }
            TextTransformKind::PrependLine { pattern, prefix } => {
                let lines: Vec<&str> = source.lines().collect();
                let result: Vec<String> = lines
                    .into_iter()
                    .map(|line| {
                        if pattern.is_match(line) {
                            format!("{prefix}{line}")
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                Ok(result.join("\n"))
            }
            TextTransformKind::AppendLine { pattern, suffix } => {
                let lines: Vec<&str> = source.lines().collect();
                let result: Vec<String> = lines
                    .into_iter()
                    .map(|line| {
                        if pattern.is_match(line) {
                            format!("{line}{suffix}")
                        } else {
                            line.to_string()
                        }
                    })
                    .collect();
                Ok(result.join("\n"))
            }
            TextTransformKind::DeleteLines { pattern } => {
                let result: Vec<&str> = source
                    .lines()
                    .filter(|line| !pattern.is_match(line))
                    .collect();
                Ok(result.join("\n"))
            }
            TextTransformKind::InsertAfter { pattern, content } => {
                let lines: Vec<&str> = source.lines().collect();
                let mut result = Vec::new();
                for line in lines {
                    result.push(line.to_string());
                    if pattern.is_match(line) {
                        result.push(content.clone());
                    }
                }
                Ok(result.join("\n"))
            }
            TextTransformKind::InsertBefore { pattern, content } => {
                let lines: Vec<&str> = source.lines().collect();
                let mut result = Vec::new();
                for line in lines {
                    if pattern.is_match(line) {
                        result.push(content.clone());
                    }
                    result.push(line.to_string());
                }
                Ok(result.join("\n"))
            }
        }
    }

    fn describe(&self) -> String {
        match &self.kind {
            TextTransformKind::Replace { pattern, replacement } => {
                format!("Replace pattern '{}' with '{}'", pattern.as_str(), replacement)
            }
            TextTransformKind::ReplaceLiteral { needle, replacement } => {
                format!("Replace literal '{}' with '{}'", needle, replacement)
            }
            TextTransformKind::PrependLine { pattern, prefix } => {
                format!("Prepend '{}' to lines matching '{}'", prefix, pattern.as_str())
            }
            TextTransformKind::AppendLine { pattern, suffix } => {
                format!("Append '{}' to lines matching '{}'", suffix, pattern.as_str())
            }
            TextTransformKind::DeleteLines { pattern } => {
                format!("Delete lines matching '{}'", pattern.as_str())
            }
            TextTransformKind::InsertAfter { pattern, .. } => {
                format!("Insert content after lines matching '{}'", pattern.as_str())
            }
            TextTransformKind::InsertBefore { pattern, .. } => {
                format!("Insert content before lines matching '{}'", pattern.as_str())
            }
        }
    }
}
