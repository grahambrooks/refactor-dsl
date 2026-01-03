//! Change signature refactoring operations.

use std::collections::HashMap;

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::{RefactorError, Result};
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::RefactoringOperation;
use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};

/// Change the signature of a function or method.
#[derive(Debug, Clone)]
pub struct ChangeSignature {
    /// Parameters to add.
    pub add_params: Vec<ParameterSpec>,
    /// Parameters to remove (by name).
    pub remove_params: Vec<String>,
    /// Parameters to rename (old_name -> new_name).
    pub rename_params: HashMap<String, String>,
    /// Parameters to reorder (list of parameter names in new order).
    pub reorder_params: Option<Vec<String>>,
    /// New return type (if changing).
    pub new_return_type: Option<String>,
    /// Whether to update call sites.
    pub update_call_sites: bool,
}

/// Specification for a new parameter.
#[derive(Debug, Clone)]
pub struct ParameterSpec {
    /// Parameter name.
    pub name: String,
    /// Parameter type.
    pub param_type: String,
    /// Default value for call sites.
    pub default_value: Option<String>,
    /// Position to insert (0 = first, -1 = last).
    pub position: i32,
}

impl ParameterSpec {
    /// Create a new parameter specification.
    pub fn new(name: impl Into<String>, param_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            param_type: param_type.into(),
            default_value: None,
            position: -1,
        }
    }

    /// Set the default value for call sites.
    pub fn with_default(mut self, value: impl Into<String>) -> Self {
        self.default_value = Some(value.into());
        self
    }

    /// Set the position to insert the parameter.
    pub fn at_position(mut self, pos: i32) -> Self {
        self.position = pos;
        self
    }
}

impl Default for ChangeSignature {
    fn default() -> Self {
        Self::new()
    }
}

impl ChangeSignature {
    /// Create a new ChangeSignature operation.
    pub fn new() -> Self {
        Self {
            add_params: Vec::new(),
            remove_params: Vec::new(),
            rename_params: HashMap::new(),
            reorder_params: None,
            new_return_type: None,
            update_call_sites: true,
        }
    }

    /// Add a new parameter.
    pub fn add_parameter(mut self, spec: ParameterSpec) -> Self {
        self.add_params.push(spec);
        self
    }

    /// Remove a parameter by name.
    pub fn remove_parameter(mut self, name: impl Into<String>) -> Self {
        self.remove_params.push(name.into());
        self
    }

    /// Rename a parameter.
    pub fn rename_parameter(
        mut self,
        old_name: impl Into<String>,
        new_name: impl Into<String>,
    ) -> Self {
        self.rename_params.insert(old_name.into(), new_name.into());
        self
    }

    /// Reorder parameters.
    pub fn reorder_parameters(mut self, order: Vec<String>) -> Self {
        self.reorder_params = Some(order);
        self
    }

    /// Change the return type.
    pub fn change_return_type(mut self, new_type: impl Into<String>) -> Self {
        self.new_return_type = Some(new_type.into());
        self
    }

    /// Skip updating call sites.
    pub fn skip_call_site_updates(mut self) -> Self {
        self.update_call_sites = false;
        self
    }

    /// Find the function at the cursor.
    fn find_function(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
    ) -> Result<Option<FunctionSignature>> {
        let tree = lang.parse(&ctx.source)?;
        let source_bytes = ctx.source.as_bytes();

        let query_str = match lang.name() {
            "rust" => {
                r#"
                (function_item
                    name: (identifier) @name
                    parameters: (parameters) @params
                    return_type: (type_identifier)? @return
                ) @func
                "#
            }
            "typescript" | "javascript" => {
                r#"
                (function_declaration
                    name: (identifier) @name
                    parameters: (formal_parameters) @params
                    return_type: (type_annotation)? @return
                ) @func
                "#
            }
            "python" => {
                r#"
                (function_definition
                    name: (identifier) @name
                    parameters: (parameters) @params
                    return_type: (type)? @return
                ) @func
                "#
            }
            "go" => {
                r#"
                (function_declaration
                    name: (identifier) @name
                    parameters: (parameter_list) @params
                    result: (_)? @return
                ) @func
                "#
            }
            "java" | "csharp" => {
                r#"
                (method_declaration
                    type: (_) @return
                    name: (identifier) @name
                    parameters: (formal_parameters) @params
                ) @func
                "#
            }
            "ruby" => {
                r#"
                (method
                    name: (identifier) @name
                    parameters: (method_parameters)? @params
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
            let mut params_text = None;
            let mut params_range = None;
            let mut return_type = None;
            let mut func_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "params" => {
                        params_text = capture.node.utf8_text(source_bytes).ok();
                        params_range = Some(Range {
                            start: Position {
                                line: capture.node.start_position().row as u32,
                                character: capture.node.start_position().column as u32,
                            },
                            end: Position {
                                line: capture.node.end_position().row as u32,
                                character: capture.node.end_position().column as u32,
                            },
                        });
                    }
                    "return" => {
                        return_type = capture
                            .node
                            .utf8_text(source_bytes)
                            .ok()
                            .map(|s| s.to_string());
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

            if let (Some(n), Some(pt), Some(pr), Some(fr)) =
                (name, params_text, params_range, func_range)
            {
                // Check if cursor is on this function
                if fr.start.line as usize <= cursor_line && cursor_line <= fr.end.line as usize {
                    let params = self.parse_parameters(pt, lang.name());
                    return Ok(Some(FunctionSignature {
                        name: n.to_string(),
                        parameters: params,
                        params_range: pr,
                        return_type,
                        function_range: fr,
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Parse parameters from parameter list text.
    fn parse_parameters(&self, params_text: &str, lang_name: &str) -> Vec<Parameter> {
        let params_text = params_text.trim();
        let inner = if params_text.starts_with('(') && params_text.ends_with(')') {
            &params_text[1..params_text.len() - 1]
        } else {
            params_text
        };

        if inner.is_empty() {
            return Vec::new();
        }

        let mut params = Vec::new();
        let mut current = String::new();
        let mut depth = 0;

        for ch in inner.chars() {
            match ch {
                '(' | '[' | '<' | '{' => {
                    depth += 1;
                    current.push(ch);
                }
                ')' | ']' | '>' | '}' => {
                    depth -= 1;
                    current.push(ch);
                }
                ',' if depth == 0 => {
                    if let Some(p) = self.parse_single_parameter(&current, lang_name) {
                        params.push(p);
                    }
                    current.clear();
                }
                _ => current.push(ch),
            }
        }

        if !current.is_empty()
            && let Some(p) = self.parse_single_parameter(&current, lang_name)
        {
            params.push(p);
        }

        params
    }

    /// Parse a single parameter.
    fn parse_single_parameter(&self, param: &str, lang_name: &str) -> Option<Parameter> {
        let param = param.trim();
        if param.is_empty() {
            return None;
        }

        match lang_name {
            "rust" => {
                // name: Type or &self, &mut self, self
                if param == "self" || param == "&self" || param == "&mut self" {
                    return Some(Parameter {
                        name: "self".to_string(),
                        param_type: param.to_string(),
                        is_self: true,
                    });
                }

                if let Some(colon_pos) = param.find(':') {
                    let name = param[..colon_pos].trim().to_string();
                    let param_type = param[colon_pos + 1..].trim().to_string();
                    Some(Parameter {
                        name,
                        param_type,
                        is_self: false,
                    })
                } else {
                    None
                }
            }
            "typescript" | "javascript" => {
                // name: Type or name
                if let Some(colon_pos) = param.find(':') {
                    let name = param[..colon_pos].trim().to_string();
                    let param_type = param[colon_pos + 1..].trim().to_string();
                    Some(Parameter {
                        name,
                        param_type,
                        is_self: false,
                    })
                } else {
                    Some(Parameter {
                        name: param.to_string(),
                        param_type: "any".to_string(),
                        is_self: false,
                    })
                }
            }
            "python" => {
                // name: Type or name or self
                if param == "self" || param == "cls" {
                    return Some(Parameter {
                        name: param.to_string(),
                        param_type: String::new(),
                        is_self: true,
                    });
                }

                if let Some(colon_pos) = param.find(':') {
                    let name = param[..colon_pos].trim().to_string();
                    let param_type = param[colon_pos + 1..].trim().to_string();
                    Some(Parameter {
                        name,
                        param_type,
                        is_self: false,
                    })
                } else {
                    Some(Parameter {
                        name: param.to_string(),
                        param_type: String::new(),
                        is_self: false,
                    })
                }
            }
            "go" => {
                // name Type or Type
                let parts: Vec<&str> = param.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(Parameter {
                        name: parts[0].to_string(),
                        param_type: parts[1..].join(" "),
                        is_self: false,
                    })
                } else if parts.len() == 1 {
                    Some(Parameter {
                        name: String::new(),
                        param_type: parts[0].to_string(),
                        is_self: false,
                    })
                } else {
                    None
                }
            }
            "java" | "csharp" => {
                // Type name
                let parts: Vec<&str> = param.split_whitespace().collect();
                if parts.len() >= 2 {
                    Some(Parameter {
                        name: parts.last()?.to_string(),
                        param_type: parts[..parts.len() - 1].join(" "),
                        is_self: false,
                    })
                } else {
                    None
                }
            }
            "ruby" => {
                // name or name = default
                let name = if let Some(eq_pos) = param.find('=') {
                    param[..eq_pos].trim().to_string()
                } else {
                    param.to_string()
                };
                Some(Parameter {
                    name,
                    param_type: String::new(),
                    is_self: false,
                })
            }
            _ => None,
        }
    }

    /// Generate new parameters list.
    fn generate_new_params(&self, current: &[Parameter], lang_name: &str) -> String {
        let mut params: Vec<Parameter> = current
            .iter()
            .filter(|p| !self.remove_params.contains(&p.name))
            .map(|p| {
                let new_name = self
                    .rename_params
                    .get(&p.name)
                    .cloned()
                    .unwrap_or_else(|| p.name.clone());
                Parameter {
                    name: new_name,
                    param_type: p.param_type.clone(),
                    is_self: p.is_self,
                }
            })
            .collect();

        // Add new parameters
        for spec in &self.add_params {
            let new_param = Parameter {
                name: spec.name.clone(),
                param_type: spec.param_type.clone(),
                is_self: false,
            };

            if spec.position < 0 || spec.position as usize >= params.len() {
                params.push(new_param);
            } else {
                params.insert(spec.position as usize, new_param);
            }
        }

        // Reorder if specified
        if let Some(ref order) = self.reorder_params {
            let mut reordered = Vec::new();
            for name in order {
                if let Some(p) = params.iter().find(|p| &p.name == name) {
                    reordered.push(p.clone());
                }
            }
            // Add any remaining parameters not in the order list
            for p in &params {
                if !order.contains(&p.name) {
                    reordered.push(p.clone());
                }
            }
            params = reordered;
        }

        // Format parameters based on language
        let param_strs: Vec<String> = params
            .iter()
            .map(|p| self.format_parameter(p, lang_name))
            .collect();

        format!("({})", param_strs.join(", "))
    }

    /// Format a single parameter.
    fn format_parameter(&self, param: &Parameter, lang_name: &str) -> String {
        if param.is_self {
            return param.param_type.clone();
        }

        match lang_name {
            "rust" => format!("{}: {}", param.name, param.param_type),
            "typescript" | "javascript" => {
                if param.param_type.is_empty() || param.param_type == "any" {
                    param.name.clone()
                } else {
                    format!("{}: {}", param.name, param.param_type)
                }
            }
            "python" => {
                if param.param_type.is_empty() {
                    param.name.clone()
                } else {
                    format!("{}: {}", param.name, param.param_type)
                }
            }
            "go" => format!("{} {}", param.name, param.param_type),
            "java" | "csharp" => format!("{} {}", param.param_type, param.name),
            "ruby" => param.name.clone(),
            _ => param.name.clone(),
        }
    }

    /// Find all call sites of the function.
    fn find_call_sites(
        &self,
        ctx: &RefactoringContext,
        lang: &dyn Language,
        func_name: &str,
    ) -> Result<Vec<CallSite>> {
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
            _ => return Ok(Vec::new()),
        };

        let query = match lang.query(query_str) {
            Ok(q) => q,
            Err(_) => return Ok(Vec::new()),
        };

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

        let mut call_sites = Vec::new();

        while let Some(m) = matches.next() {
            let mut name = None;
            let mut args_text = None;
            let mut args_range = None;

            for capture in m.captures {
                let capture_name = query.capture_names()[capture.index as usize];
                match capture_name {
                    "name" => {
                        name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "args" => {
                        args_text = capture.node.utf8_text(source_bytes).ok();
                        args_range = Some(Range {
                            start: Position {
                                line: capture.node.start_position().row as u32,
                                character: capture.node.start_position().column as u32,
                            },
                            end: Position {
                                line: capture.node.end_position().row as u32,
                                character: capture.node.end_position().column as u32,
                            },
                        });
                    }
                    _ => {}
                }
            }

            if let (Some(n), Some(at), Some(ar)) = (name, args_text, args_range)
                && n == func_name
            {
                call_sites.push(CallSite {
                    args_text: at.to_string(),
                    args_range: ar,
                });
            }
        }

        Ok(call_sites)
    }
}

/// Information about a function signature.
#[derive(Debug, Clone)]
struct FunctionSignature {
    name: String,
    parameters: Vec<Parameter>,
    params_range: Range,
    #[allow(dead_code)]
    return_type: Option<String>,
    #[allow(dead_code)]
    function_range: Range,
}

/// A parameter in a function signature.
#[derive(Debug, Clone)]
struct Parameter {
    name: String,
    param_type: String,
    is_self: bool,
}

/// A call site of a function.
#[derive(Debug, Clone)]
struct CallSite {
    #[allow(dead_code)]
    args_text: String,
    args_range: Range,
}

impl RefactoringOperation for ChangeSignature {
    fn name(&self) -> &'static str {
        "Change Signature"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let func = self.find_function(ctx, lang)?;
        if func.is_none() {
            return Ok(ValidationResult::invalid(
                "No function found at cursor position",
            ));
        }

        let func = func.unwrap();

        // Check that removed parameters exist
        for name in &self.remove_params {
            if !func.parameters.iter().any(|p| &p.name == name) {
                return Ok(ValidationResult::invalid(format!(
                    "Parameter '{}' not found in function signature",
                    name
                )));
            }
        }

        // Check that renamed parameters exist
        for old_name in self.rename_params.keys() {
            if !func.parameters.iter().any(|p| &p.name == old_name) {
                return Ok(ValidationResult::invalid(format!(
                    "Parameter '{}' not found in function signature",
                    old_name
                )));
            }
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let lang = ctx
            .language()
            .ok_or_else(|| RefactorError::InvalidConfig("No language detected".to_string()))?;

        let func = self
            .find_function(ctx, lang)?
            .ok_or_else(|| RefactorError::InvalidConfig("No function found".to_string()))?;

        let mut preview = RefactoringPreview::new(format!("Change signature of '{}'", func.name));

        // Generate new parameter list
        let new_params = self.generate_new_params(&func.parameters, lang.name());

        // Edit the function signature
        preview.add_edit(TextEdit::new(
            ctx.target_file.clone(),
            func.params_range,
            new_params.clone(),
        ));

        // Find and update call sites
        if self.update_call_sites {
            let call_sites = self.find_call_sites(ctx, lang, &func.name)?;
            for call in &call_sites {
                // Update call site arguments
                // This is simplified - a full implementation would need to
                // track argument positions and handle defaults
                let new_args = self.update_call_args(&call.args_text, lang.name());
                preview.add_edit(TextEdit::new(
                    ctx.target_file.clone(),
                    call.args_range,
                    new_args,
                ));
            }
        }

        let mut changes = Vec::new();
        if !self.add_params.is_empty() {
            changes.push(format!(
                "add: {:?}",
                self.add_params.iter().map(|p| &p.name).collect::<Vec<_>>()
            ));
        }
        if !self.remove_params.is_empty() {
            changes.push(format!("remove: {:?}", self.remove_params));
        }
        if !self.rename_params.is_empty() {
            changes.push(format!("rename: {:?}", self.rename_params));
        }

        let diff = format!(
            "Change signature of '{}'\nOld: {}\nNew: {}\nChanges: {}",
            func.name,
            func.parameters
                .iter()
                .map(|p| p.name.as_str())
                .collect::<Vec<_>>()
                .join(", "),
            new_params,
            changes.join(", ")
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

        Ok(RefactoringResult::success("Changed function signature")
            .with_file(ctx.target_file.clone())
            .with_edits(edits))
    }
}

impl ChangeSignature {
    /// Update call site arguments based on signature changes.
    fn update_call_args(&self, args_text: &str, _lang_name: &str) -> String {
        // This is a simplified implementation
        // A full implementation would parse the arguments and handle:
        // - Removing arguments for removed parameters
        // - Adding default values for new parameters
        // - Reordering arguments

        let inner = args_text.trim();
        let inner = if inner.starts_with('(') && inner.ends_with(')') {
            &inner[1..inner.len() - 1]
        } else {
            inner
        };

        let mut args: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

        // Add default values for new parameters
        for spec in &self.add_params {
            if let Some(ref default) = spec.default_value {
                if spec.position < 0 || spec.position as usize >= args.len() {
                    args.push(default);
                } else {
                    args.insert(spec.position as usize, default);
                }
            }
        }

        format!("({})", args.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_change_signature_validation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn hello(name: &str) { println!(\"{}\", name); }")
            .with_selection(0, 0, 0, 45);

        let op = ChangeSignature::new()
            .add_parameter(ParameterSpec::new("greeting", "&str").with_default("\"Hello\""));

        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_change_signature_remove_nonexistent() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn hello(name: &str) {}")
            .with_selection(0, 0, 0, 23);

        let op = ChangeSignature::new().remove_parameter("nonexistent");

        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_parse_rust_parameters() {
        let op = ChangeSignature::new();

        let params = op.parse_parameters("(name: &str, count: i32)", "rust");
        assert_eq!(params.len(), 2);
        assert_eq!(params[0].name, "name");
        assert_eq!(params[0].param_type, "&str");
        assert_eq!(params[1].name, "count");
        assert_eq!(params[1].param_type, "i32");
    }

    #[test]
    fn test_parse_rust_self_parameter() {
        let op = ChangeSignature::new();

        let params = op.parse_parameters("(&self, name: &str)", "rust");
        assert_eq!(params.len(), 2);
        assert!(params[0].is_self);
        assert_eq!(params[1].name, "name");
    }

    #[test]
    fn test_parse_python_parameters() {
        let op = ChangeSignature::new();

        let params = op.parse_parameters("(self, name: str, count: int)", "python");
        assert_eq!(params.len(), 3);
        assert!(params[0].is_self);
        assert_eq!(params[1].name, "name");
        assert_eq!(params[2].name, "count");
    }

    #[test]
    fn test_generate_new_params() {
        let op = ChangeSignature::new().add_parameter(ParameterSpec::new("extra", "bool"));

        let current = vec![Parameter {
            name: "name".to_string(),
            param_type: "&str".to_string(),
            is_self: false,
        }];

        let new_params = op.generate_new_params(&current, "rust");
        assert!(new_params.contains("name: &str"));
        assert!(new_params.contains("extra: bool"));
    }

    #[test]
    fn test_generate_params_with_remove() {
        let op = ChangeSignature::new().remove_parameter("count");

        let current = vec![
            Parameter {
                name: "name".to_string(),
                param_type: "&str".to_string(),
                is_self: false,
            },
            Parameter {
                name: "count".to_string(),
                param_type: "i32".to_string(),
                is_self: false,
            },
        ];

        let new_params = op.generate_new_params(&current, "rust");
        assert!(new_params.contains("name: &str"));
        assert!(!new_params.contains("count"));
    }
}
