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

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_path() -> &'static Path {
        Path::new("test.rs")
    }

    #[test]
    fn test_replace_pattern() {
        let transform = TextTransform::replace(r"\.unwrap\(\)", ".expect(\"error\")");
        let source = "let x = foo().unwrap();";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "let x = foo().expect(\"error\");");
    }

    #[test]
    fn test_replace_pattern_multiple() {
        let transform = TextTransform::replace(r"old", "new");
        let source = "old_func old_var old_type";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "new_func new_var new_type");
    }

    #[test]
    fn test_replace_pattern_with_groups() {
        let transform = TextTransform::replace(r"fn (\w+)", "pub fn $1");
        let source = "fn hello() {}";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "pub fn hello() {}");
    }

    #[test]
    fn test_replace_literal() {
        let transform = TextTransform::replace_literal("old_name", "new_name");
        let source = "use old_name::module;";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "use new_name::module;");
    }

    #[test]
    fn test_replace_literal_no_regex() {
        // Literal replacement should not interpret regex special chars
        let transform = TextTransform::replace_literal(".*", "STAR");
        let source = "match .* pattern";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "match STAR pattern");
    }

    #[test]
    fn test_prepend_line() {
        let transform = TextTransform::prepend_line(r"^\s*fn ", "// TODO: document\n").unwrap();
        let source = "fn hello() {}\nlet x = 1;\nfn world() {}";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert!(result.contains("// TODO: document\nfn hello"));
        assert!(result.contains("// TODO: document\nfn world"));
        assert!(!result.contains("// TODO: document\nlet"));
    }

    #[test]
    fn test_append_line() {
        let transform = TextTransform::append_line(r";\s*$", " // added").unwrap();
        let source = "let x = 1;\nfn foo() {}\nlet y = 2;";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert!(result.contains("let x = 1; // added"));
        assert!(result.contains("let y = 2; // added"));
        assert!(!result.contains("fn foo() {} // added"));
    }

    #[test]
    fn test_delete_lines() {
        let transform = TextTransform::delete_lines(r"^\s*//").unwrap();
        let source = "// comment\nlet x = 1;\n// another comment\nlet y = 2;";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert!(!result.contains("// comment"));
        assert!(result.contains("let x = 1;"));
        assert!(result.contains("let y = 2;"));
    }

    #[test]
    fn test_insert_after() {
        let transform = TextTransform::insert_after(r"^use ", "// imported").unwrap();
        let source = "use std::io;\nuse std::fs;\nfn main() {}";
        let result = transform.apply(source, dummy_path()).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "use std::io;");
        assert_eq!(lines[1], "// imported");
        assert_eq!(lines[2], "use std::fs;");
        assert_eq!(lines[3], "// imported");
    }

    #[test]
    fn test_insert_before() {
        let transform = TextTransform::insert_before(r"^fn ", "#[inline]").unwrap();
        let source = "fn hello() {}\nlet x = 1;\nfn world() {}";
        let result = transform.apply(source, dummy_path()).unwrap();
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines[0], "#[inline]");
        assert_eq!(lines[1], "fn hello() {}");
    }

    #[test]
    fn test_no_match() {
        let transform = TextTransform::replace(r"xyz", "abc");
        let source = "hello world";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, source);
    }

    #[test]
    fn test_empty_source() {
        let transform = TextTransform::replace(r"foo", "bar");
        let source = "";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "");
    }

    #[test]
    fn test_describe_replace() {
        let transform = TextTransform::replace(r"old", "new");
        let desc = transform.describe();
        assert!(desc.contains("old"));
        assert!(desc.contains("new"));
    }

    #[test]
    fn test_describe_literal() {
        let transform = TextTransform::replace_literal("old", "new");
        let desc = transform.describe();
        assert!(desc.contains("literal"));
    }

    #[test]
    fn test_describe_delete() {
        let transform = TextTransform::delete_lines(r"comment").unwrap();
        let desc = transform.describe();
        assert!(desc.contains("Delete"));
        assert!(desc.contains("comment"));
    }

    #[test]
    fn test_replace_regex_precompiled() {
        let pattern = Regex::new(r"\d+").unwrap();
        let transform = TextTransform::replace_regex(pattern, "NUM");
        let source = "x = 42, y = 123";
        let result = transform.apply(source, dummy_path()).unwrap();
        assert_eq!(result, "x = NUM, y = NUM");
    }
}
