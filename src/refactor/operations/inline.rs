//! Inline refactoring operations.

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::{RefactorError, Result};
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::RefactoringOperation;
use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};

/// Inline a variable - replace all usages with its value.
#[derive(Debug, Clone)]
pub struct InlineVariable {
    /// Whether to delete the variable declaration after inlining.
    pub delete_declaration: bool,
}

impl Default for InlineVariable {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineVariable {
    /// Create a new InlineVariable operation.
    pub fn new() -> Self {
        Self {
            delete_declaration: true,
        }
    }

    /// Keep the variable declaration after inlining.
    pub fn keep_declaration(mut self) -> Self {
        self.delete_declaration = false;
        self
    }

    /// Find variable declaration at the cursor position.
    fn find_declaration(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<VariableInfo>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        // Query for variable declarations based on language
        let query_str = match lang.name() {
            "rust" => {
                r#"
                (let_declaration
                    pattern: (identifier) @name
                    value: (_) @value
                ) @decl
                "#
            }
            "typescript" | "javascript" => {
                r#"
                (variable_declarator
                    name: (identifier) @name
                    value: (_) @value
                ) @decl
                "#
            }
            "python" => {
                r#"
                (assignment
                    left: (identifier) @name
                    right: (_) @value
                ) @decl
                "#
            }
            "go" => {
                r#"
                (short_var_declaration
                    left: (expression_list (identifier) @name)
                    right: (expression_list (_) @value)
                ) @decl
                "#
            }
            "java" | "csharp" => {
                r#"
                (variable_declarator
                    name: (identifier) @name
                    value: (_) @value
                ) @decl
                "#
            }
            "ruby" => {
                r#"
                (assignment
                    left: (identifier) @name
                    right: (_) @value
                ) @decl
                "#
            }
            _ => return Ok(None),
        };

        let query = lang.query(query_str)?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let cursor_line = ctx.target_range.start.line as usize;

        while let Some(m) = matches.next() {
            let mut name = None;
            let mut value = None;
            let mut decl_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "value" => {
                        value = capture.node.utf8_text(source_bytes).ok();
                    }
                    "decl" => {
                        let node = capture.node;
                        decl_range = Some(Range {
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

            if let (Some(n), Some(v), Some(range)) = (name, value, decl_range) {
                // Check if cursor is on this declaration
                if range.start.line as usize <= cursor_line
                    && cursor_line <= range.end.line as usize
                {
                    return Ok(Some(VariableInfo {
                        name: n.to_string(),
                        value: v.to_string(),
                        declaration_range: range,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Find all usages of a variable.
    fn find_usages(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
        var_name: &str,
        declaration_range: &Range,
    ) -> Result<Vec<Range>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query = lang.query("(identifier) @id")?;
        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut usages = Vec::new();

        while let Some(m) = matches.next() {
            for capture in m.captures {
                if let Ok(text) = capture.node.utf8_text(source_bytes)
                    && text == var_name
                {
                    let range = Range {
                        start: Position {
                            line: capture.node.start_position().row as u32,
                            character: capture.node.start_position().column as u32,
                        },
                        end: Position {
                            line: capture.node.end_position().row as u32,
                            character: capture.node.end_position().column as u32,
                        },
                    };

                    // Skip the declaration itself
                    if range.start.line != declaration_range.start.line
                        || range.start.character != declaration_range.start.character
                    {
                        // Skip if this is within the declaration range (the name in the declaration)
                        if range.start.line < declaration_range.start.line
                            || range.start.line > declaration_range.end.line
                            || (range.start.line == declaration_range.start.line
                                && range.start.character < declaration_range.start.character)
                            || (range.start.line == declaration_range.end.line
                                && range.start.character >= declaration_range.end.character)
                        {
                            // Only include usages after the declaration
                            if range.start.line > declaration_range.end.line
                                || (range.start.line == declaration_range.end.line
                                    && range.start.character > declaration_range.end.character)
                            {
                                usages.push(range);
                            }
                        }
                    }
                }
            }
        }

        Ok(usages)
    }
}

/// Information about a variable.
#[derive(Debug, Clone)]
struct VariableInfo {
    name: String,
    value: String,
    declaration_range: Range,
}

impl RefactoringOperation for InlineVariable {
    fn name(&self) -> &'static str {
        "Inline Variable"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let var_info = self.find_declaration(ctx, lang)?;
        if var_info.is_none() {
            return Ok(ValidationResult::invalid(
                "No variable declaration found at cursor position",
            ));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let var_info = self.find_declaration(ctx, lang)?.ok_or_else(|| {
            RefactorError::InvalidConfig("No variable declaration found".to_string())
        })?;

        let mut preview = RefactoringPreview::new(format!(
            "Inline variable '{}' with value '{}'",
            var_info.name, var_info.value
        ));

        // Find all usages
        let usages = self.find_usages(ctx, lang, &var_info.name, &var_info.declaration_range)?;

        // Replace each usage with the value
        // May need to wrap in parentheses for complex expressions
        let needs_parens = var_info.value.contains(' ')
            && !var_info.value.starts_with('(')
            && !var_info.value.starts_with('"')
            && !var_info.value.starts_with('\'');

        let replacement = if needs_parens {
            format!("({})", var_info.value)
        } else {
            var_info.value.clone()
        };

        for usage in &usages {
            preview.add_edit(TextEdit::new(
                ctx.target_file.clone(),
                *usage,
                replacement.clone(),
            ));
        }

        // Delete the declaration if requested
        if self.delete_declaration {
            // Extend range to include the whole line if it's the only thing on the line
            let line_start = Position {
                line: var_info.declaration_range.start.line,
                character: 0,
            };
            let line_end = Position {
                line: var_info.declaration_range.end.line + 1,
                character: 0,
            };

            preview.add_edit(TextEdit::new(
                ctx.target_file.clone(),
                Range {
                    start: line_start,
                    end: line_end,
                },
                String::new(),
            ));
        }

        let diff = format!(
            "Inline '{}' ({} usage(s)) with: {}{}",
            var_info.name,
            usages.len(),
            var_info.value,
            if self.delete_declaration {
                "\nDelete declaration"
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

        Ok(RefactoringResult::success("Inlined variable")
            .with_file(ctx.target_file.clone())
            .with_edits(edits))
    }
}

/// Inline a function - replace all calls with its body.
#[derive(Debug, Clone)]
pub struct InlineFunction {
    /// Whether to delete the function after inlining.
    pub delete_function: bool,
}

impl Default for InlineFunction {
    fn default() -> Self {
        Self::new()
    }
}

impl InlineFunction {
    /// Create a new InlineFunction operation.
    pub fn new() -> Self {
        Self {
            delete_function: false,
        }
    }

    /// Delete the function after inlining all calls.
    pub fn delete_after_inline(mut self) -> Self {
        self.delete_function = true;
        self
    }

    /// Find function definition at the cursor position.
    fn find_function(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<FunctionInfo>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => {
                r#"
                (function_item
                    name: (identifier) @name
                    parameters: (parameters) @params
                    body: (block) @body
                ) @func
                "#
            }
            "typescript" | "javascript" => {
                r#"
                (function_declaration
                    name: (identifier) @name
                    parameters: (formal_parameters) @params
                    body: (statement_block) @body
                ) @func
                "#
            }
            "python" => {
                r#"
                (function_definition
                    name: (identifier) @name
                    parameters: (parameters) @params
                    body: (block) @body
                ) @func
                "#
            }
            "go" => {
                r#"
                (function_declaration
                    name: (identifier) @name
                    parameters: (parameter_list) @params
                    body: (block) @body
                ) @func
                "#
            }
            "ruby" => {
                r#"
                (method
                    name: (identifier) @name
                    parameters: (method_parameters)? @params
                    body: (_)* @body
                ) @func
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
            let mut params = None;
            let mut body = None;
            let mut func_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "params" => {
                        params = capture.node.utf8_text(source_bytes).ok();
                    }
                    "body" => {
                        body = capture.node.utf8_text(source_bytes).ok();
                    }
                    "func" => {
                        let node = capture.node;
                        func_range = Some(Range {
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

            if let (Some(n), Some(b), Some(range)) = (name, body, func_range) {
                // Check if cursor is on this function
                if range.start.line as usize <= cursor_line
                    && cursor_line <= range.end.line as usize
                {
                    return Ok(Some(FunctionInfo {
                        name: n.to_string(),
                        params: params.unwrap_or("").to_string(),
                        body: b.to_string(),
                        function_range: range,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Find all calls to a function.
    fn find_calls(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
        func_name: &str,
    ) -> Result<Vec<CallInfo>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => {
                "(call_expression function: (identifier) @name arguments: (arguments) @args) @call"
            }
            "typescript" | "javascript" => {
                "(call_expression function: (identifier) @name arguments: (arguments) @args) @call"
            }
            "python" => {
                "(call function: (identifier) @name arguments: (argument_list) @args) @call"
            }
            "go" => {
                "(call_expression function: (identifier) @name arguments: (argument_list) @args) @call"
            }
            "ruby" => "(call method: (identifier) @name arguments: (argument_list)? @args) @call",
            _ => return Ok(Vec::new()),
        };

        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut calls = Vec::new();

        while let Some(m) = matches.next() {
            let mut name = None;
            let mut args = None;
            let mut call_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "args" => {
                        args = capture.node.utf8_text(source_bytes).ok();
                    }
                    "call" => {
                        let node = capture.node;
                        call_range = Some(Range {
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

            if let (Some(n), Some(range)) = (name, call_range)
                && n == func_name
            {
                calls.push(CallInfo {
                    args: args.unwrap_or("()").to_string(),
                    range,
                });
            }
        }

        Ok(calls)
    }

    /// Extract the inner body (without braces).
    fn extract_body_content(&self, body: &str, lang_name: &str) -> String {
        let body = body.trim();

        match lang_name {
            "rust" | "go" | "java" | "csharp" | "typescript" | "javascript" => {
                // Remove surrounding braces
                if body.starts_with('{') && body.ends_with('}') {
                    let inner = &body[1..body.len() - 1];
                    // Find return statement and extract expression
                    if let Some(ret_idx) = inner.find("return ") {
                        let after_return = &inner[ret_idx + 7..];
                        if let Some(semi_idx) = after_return.find(';') {
                            return after_return[..semi_idx].trim().to_string();
                        }
                        return after_return.trim().trim_end_matches(';').to_string();
                    }
                    inner.trim().to_string()
                } else {
                    body.to_string()
                }
            }
            "python" => {
                // Find return statement
                for line in body.lines() {
                    let trimmed = line.trim();
                    if let Some(stripped) = trimmed.strip_prefix("return ") {
                        return stripped.trim().to_string();
                    }
                }
                body.trim().to_string()
            }
            "ruby" => {
                // Ruby implicitly returns last expression
                body.lines()
                    .last()
                    .map(|l| l.trim().to_string())
                    .unwrap_or_default()
            }
            _ => body.to_string(),
        }
    }
}

/// Information about a function.
#[derive(Debug, Clone)]
struct FunctionInfo {
    name: String,
    #[allow(dead_code)] // Reserved for parameter substitution
    params: String,
    body: String,
    function_range: Range,
}

/// Information about a function call.
#[derive(Debug, Clone)]
struct CallInfo {
    #[allow(dead_code)] // Reserved for parameter substitution
    args: String,
    range: Range,
}

impl RefactoringOperation for InlineFunction {
    fn name(&self) -> &'static str {
        "Inline Function"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let func_info = self.find_function(ctx, lang)?;
        if func_info.is_none() {
            return Ok(ValidationResult::invalid(
                "No function definition found at cursor position",
            ));
        }

        let func_info = func_info.unwrap();

        // Check if function has calls
        let calls = self.find_calls(ctx, lang, &func_info.name)?;
        if calls.is_empty() {
            return Ok(ValidationResult::invalid("No calls to this function found")
                .with_warning("Function has no usages to inline"));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let func_info = self.find_function(ctx, lang)?.ok_or_else(|| {
            RefactorError::InvalidConfig("No function definition found".to_string())
        })?;

        let mut preview = RefactoringPreview::new(format!(
            "Inline function '{}' at all call sites",
            func_info.name
        ));

        // Find all calls
        let calls = self.find_calls(ctx, lang, &func_info.name)?;

        // Get the body content to inline
        let body_content = self.extract_body_content(&func_info.body, lang.name());

        // Replace each call with the body
        for call in &calls {
            // For simple cases, just replace the call with the body
            // TODO: Handle parameter substitution
            preview.add_edit(TextEdit::new(
                ctx.target_file.clone(),
                call.range,
                body_content.clone(),
            ));
        }

        // Delete the function if requested
        if self.delete_function {
            let line_start = Position {
                line: func_info.function_range.start.line,
                character: 0,
            };
            let line_end = Position {
                line: func_info.function_range.end.line + 1,
                character: 0,
            };

            preview.add_edit(TextEdit::new(
                ctx.target_file.clone(),
                Range {
                    start: line_start,
                    end: line_end,
                },
                String::new(),
            ));
        }

        let diff = format!(
            "Inline '{}' at {} call site(s) with: {}{}",
            func_info.name,
            calls.len(),
            body_content,
            if self.delete_function {
                "\nDelete function definition"
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

        Ok(RefactoringResult::success("Inlined function")
            .with_file(ctx.target_file.clone())
            .with_edits(edits))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inline_variable_validation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let x = 42;\n    println!(\"{}\", x);\n}")
            .with_selection(1, 4, 1, 14);

        let op = InlineVariable::new();
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_inline_variable_no_declaration() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    println!(\"hello\");\n}")
            .with_selection(1, 4, 1, 10);

        let op = InlineVariable::new();
        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_extract_body_content_rust() {
        let op = InlineFunction::new();

        let body = "{ return x + y; }";
        assert_eq!(op.extract_body_content(body, "rust"), "x + y");

        let body = "{ x + y }";
        assert_eq!(op.extract_body_content(body, "rust"), "x + y");
    }

    #[test]
    fn test_extract_body_content_python() {
        let op = InlineFunction::new();

        let body = "    return x + y";
        assert_eq!(op.extract_body_content(body, "python"), "x + y");
    }
}
