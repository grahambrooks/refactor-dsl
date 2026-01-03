//! Extract refactoring operations.

use std::collections::HashSet;

use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use crate::error::Result;
use crate::lang::Language;
use crate::lsp::{Position, Range};

use super::RefactoringOperation;
use super::context::{
    RefactoringContext, RefactoringPreview, RefactoringResult, TextEdit, ValidationResult,
};

/// Visibility level for extracted items.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Visibility {
    /// Private (default).
    #[default]
    Private,
    /// Public.
    Public,
    /// Protected (for languages that support it).
    Protected,
}

impl Visibility {
    /// Get the visibility keyword for a language.
    pub fn keyword(&self, lang_name: &str) -> &'static str {
        match (self, lang_name) {
            (Visibility::Private, "rust") => "",
            (Visibility::Public, "rust") => "pub ",
            (Visibility::Private, "go") => "",
            (Visibility::Public, "go") => "", // Go uses capitalization
            (Visibility::Private, "java" | "csharp") => "private ",
            (Visibility::Public, "java" | "csharp") => "public ",
            (Visibility::Protected, "java" | "csharp") => "protected ",
            (Visibility::Private, "typescript") => "",
            (Visibility::Public, "typescript") => "export ",
            (Visibility::Private, "python") => "_",
            (Visibility::Public, "python") => "",
            (Visibility::Private, "ruby") => "",
            (Visibility::Public, "ruby") => "",
            _ => "",
        }
    }
}

/// Strategy for inferring parameters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ParameterStrategy {
    /// Automatically infer parameters from used variables.
    #[default]
    Infer,
    /// Use explicitly provided parameters.
    Explicit,
}

/// Extract a selection into a new function.
#[derive(Debug, Clone)]
pub struct ExtractFunction {
    /// Name for the new function.
    pub name: String,
    /// Visibility of the new function.
    pub visibility: Visibility,
    /// Parameter inference strategy.
    pub parameter_strategy: ParameterStrategy,
    /// Explicitly provided parameters (if strategy is Explicit).
    pub explicit_params: Vec<(String, String)>, // (name, type)
    /// Whether to extract as async function.
    pub is_async: bool,
}

impl ExtractFunction {
    /// Create a new ExtractFunction operation.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visibility: Visibility::default(),
            parameter_strategy: ParameterStrategy::default(),
            explicit_params: Vec::new(),
            is_async: false,
        }
    }

    /// Set visibility.
    pub fn with_visibility(mut self, visibility: Visibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Make the function public.
    pub fn public(self) -> Self {
        self.with_visibility(Visibility::Public)
    }

    /// Set parameter strategy.
    pub fn with_parameter_strategy(mut self, strategy: ParameterStrategy) -> Self {
        self.parameter_strategy = strategy;
        self
    }

    /// Add explicit parameters.
    pub fn with_params(mut self, params: Vec<(String, String)>) -> Self {
        self.explicit_params = params;
        self.parameter_strategy = ParameterStrategy::Explicit;
        self
    }

    /// Make the function async.
    pub fn async_fn(mut self) -> Self {
        self.is_async = true;
        self
    }

    /// Infer parameters from the selected code.
    fn infer_parameters(
        &self,
        ctx: &RefactoringContext,
        selected_code: &str,
    ) -> Result<Vec<(String, Option<String>)>> {
        let lang = ctx.language().ok_or_else(|| {
            crate::error::RefactorError::InvalidConfig("No language detected".to_string())
        })?;

        let mut params = Vec::new();
        let used_vars = self.find_used_identifiers(selected_code, lang)?;

        // For each used identifier, try to find its type from the scope
        for var_name in used_vars {
            // Simple heuristic: check if this looks like a parameter name
            // (not a function call, not a keyword, etc.)
            if self.is_likely_variable(&var_name, lang.name()) {
                params.push((var_name, None));
            }
        }

        Ok(params)
    }

    /// Find identifiers used in the selected code.
    fn find_used_identifiers(&self, code: &str, lang: &dyn Language) -> Result<HashSet<String>> {
        let tree = lang.parse(code)?;
        let source_bytes = code.as_bytes();
        let mut identifiers = HashSet::new();

        // Use a query to find all identifiers
        let query_str = match lang.name() {
            "rust" => "(identifier) @id",
            "typescript" | "javascript" => "(identifier) @id",
            "python" => "(identifier) @id",
            "go" => "(identifier) @id",
            "java" => "(identifier) @id",
            "csharp" => "(identifier) @id",
            "ruby" => "(identifier) @id",
            _ => return Ok(identifiers),
        };

        if let Ok(query) = lang.query(query_str) {
            let mut cursor = QueryCursor::new();
            let mut matches = cursor.matches(&query, tree.root_node(), source_bytes);

            while let Some(m) = matches.next() {
                for capture in m.captures {
                    if let Ok(text) = capture.node.utf8_text(source_bytes) {
                        identifiers.insert(text.to_string());
                    }
                }
            }
        }

        Ok(identifiers)
    }

    /// Check if an identifier is likely a variable (not a function, keyword, etc.).
    fn is_likely_variable(&self, name: &str, lang_name: &str) -> bool {
        // Filter out common keywords and patterns
        let keywords: &[&str] = match lang_name {
            "rust" => &[
                "fn", "let", "mut", "if", "else", "for", "while", "loop", "match", "return",
                "self", "Self", "true", "false", "pub", "use", "mod", "struct", "enum", "impl",
                "trait", "where", "async", "await", "move", "ref", "static", "const", "type",
                "dyn", "unsafe",
            ],
            "typescript" | "javascript" => &[
                "function",
                "let",
                "const",
                "var",
                "if",
                "else",
                "for",
                "while",
                "return",
                "this",
                "true",
                "false",
                "null",
                "undefined",
                "class",
                "interface",
                "type",
                "export",
                "import",
                "async",
                "await",
                "new",
                "typeof",
                "instanceof",
            ],
            "python" => &[
                "def", "class", "if", "else", "elif", "for", "while", "return", "self", "True",
                "False", "None", "import", "from", "as", "try", "except", "finally", "with",
                "lambda", "yield", "async", "await", "pass", "break", "continue",
            ],
            "go" => &[
                "func",
                "var",
                "const",
                "if",
                "else",
                "for",
                "range",
                "return",
                "true",
                "false",
                "nil",
                "type",
                "struct",
                "interface",
                "package",
                "import",
                "go",
                "defer",
                "select",
                "chan",
                "map",
                "make",
                "new",
                "append",
                "len",
                "cap",
            ],
            "java" => &[
                "class",
                "interface",
                "public",
                "private",
                "protected",
                "static",
                "final",
                "void",
                "if",
                "else",
                "for",
                "while",
                "return",
                "this",
                "super",
                "new",
                "true",
                "false",
                "null",
                "try",
                "catch",
                "finally",
                "throw",
                "throws",
                "import",
                "package",
            ],
            "csharp" => &[
                "class",
                "interface",
                "public",
                "private",
                "protected",
                "internal",
                "static",
                "void",
                "if",
                "else",
                "for",
                "foreach",
                "while",
                "return",
                "this",
                "base",
                "new",
                "true",
                "false",
                "null",
                "try",
                "catch",
                "finally",
                "throw",
                "using",
                "namespace",
                "async",
                "await",
                "var",
            ],
            "ruby" => &[
                "def",
                "class",
                "module",
                "if",
                "else",
                "elsif",
                "unless",
                "for",
                "while",
                "until",
                "return",
                "self",
                "true",
                "false",
                "nil",
                "do",
                "end",
                "begin",
                "rescue",
                "ensure",
                "yield",
                "super",
                "require",
                "include",
                "extend",
                "attr_reader",
                "attr_writer",
                "attr_accessor",
            ],
            _ => &[],
        };

        !keywords.contains(&name)
            && !name.is_empty()
            && name
                .chars()
                .next()
                .is_some_and(|c| c.is_alphabetic() || c == '_')
    }

    /// Generate the function signature.
    fn generate_signature(
        &self,
        lang_name: &str,
        params: &[(String, Option<String>)],
        return_type: Option<&str>,
    ) -> String {
        let vis = self.visibility.keyword(lang_name);
        let async_kw = if self.is_async {
            match lang_name {
                "rust" => "async ",
                "typescript" | "javascript" => "async ",
                "python" => "async ",
                "csharp" => "async ",
                _ => "",
            }
        } else {
            ""
        };

        let name = if lang_name == "go" && self.visibility == Visibility::Public {
            // Go uses capitalization for visibility
            let mut chars = self.name.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
                None => self.name.clone(),
            }
        } else if lang_name == "python" && self.visibility == Visibility::Private {
            format!("_{}", self.name)
        } else {
            self.name.clone()
        };

        let param_str = self.format_params(lang_name, params);

        match lang_name {
            "rust" => {
                let ret = return_type
                    .map(|t| format!(" -> {}", t))
                    .unwrap_or_default();
                format!("{}{}fn {}({}){}", vis, async_kw, name, param_str, ret)
            }
            "typescript" | "javascript" => {
                let ret = return_type.map(|t| format!(": {}", t)).unwrap_or_default();
                format!("{}{}function {}({}){}", vis, async_kw, name, param_str, ret)
            }
            "python" => {
                let ret = return_type
                    .map(|t| format!(" -> {}", t))
                    .unwrap_or_default();
                format!("{}def {}({}){}:", async_kw, name, param_str, ret)
            }
            "go" => {
                let ret = return_type.map(|t| format!(" {}", t)).unwrap_or_default();
                format!("func {}({}){}", name, param_str, ret)
            }
            "java" => {
                let ret = return_type.unwrap_or("void");
                format!("{}{} {}({})", vis, ret, name, param_str)
            }
            "csharp" => {
                let ret = return_type.unwrap_or("void");
                format!("{}{}{} {}({})", vis, async_kw, ret, name, param_str)
            }
            "ruby" => {
                format!("def {}({})", name, param_str)
            }
            _ => format!("function {}({})", name, param_str),
        }
    }

    /// Format parameters for a language.
    fn format_params(&self, lang_name: &str, params: &[(String, Option<String>)]) -> String {
        params
            .iter()
            .map(|(name, ty)| match (lang_name, ty) {
                ("rust", Some(t)) => format!("{}: {}", name, t),
                ("rust", None) => format!("{}: _", name),
                ("typescript", Some(t)) => format!("{}: {}", name, t),
                ("typescript", None) => name.clone(),
                ("python", Some(t)) => format!("{}: {}", name, t),
                ("python", None) => name.clone(),
                ("go", Some(t)) => format!("{} {}", name, t),
                ("go", None) => name.clone(),
                ("java" | "csharp", Some(t)) => format!("{} {}", t, name),
                ("java" | "csharp", None) => format!("Object {}", name),
                ("ruby", _) => name.clone(),
                _ => name.clone(),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Generate the function body wrapper.
    fn wrap_body(&self, lang_name: &str, body: &str, indent: &str) -> String {
        let body_indent = format!("{}    ", indent);
        let indented_body = body
            .lines()
            .map(|line| {
                if line.trim().is_empty() {
                    line.to_string()
                } else {
                    format!("{}{}", body_indent, line.trim())
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        match lang_name {
            "rust" | "go" | "java" | "csharp" | "typescript" | "javascript" => {
                format!(" {{\n{}\n{}}}", indented_body, indent)
            }
            "python" => {
                format!("\n{}", indented_body)
            }
            "ruby" => {
                format!("\n{}\n{}end", indented_body, indent)
            }
            _ => format!(" {{\n{}\n{}}}", indented_body, indent),
        }
    }
}

impl RefactoringOperation for ExtractFunction {
    fn name(&self) -> &'static str {
        "Extract Function"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let selected = ctx.selected_text();
        if selected.trim().is_empty() {
            return Ok(ValidationResult::invalid("No code selected"));
        }

        if self.name.is_empty() {
            return Ok(ValidationResult::invalid("Function name is required"));
        }

        // Check for valid identifier
        if !self
            .name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Ok(ValidationResult::invalid(
                "Function name must start with a letter or underscore",
            ));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let mut preview =
            RefactoringPreview::new(format!("Extract selection into function '{}'", self.name));

        let lang = ctx.language().ok_or_else(|| {
            crate::error::RefactorError::InvalidConfig("No language detected".to_string())
        })?;

        let selected = ctx.selected_text();
        let params = self.infer_parameters(ctx, selected)?;
        let indent = ctx.get_indentation(ctx.target_range.start.line);

        // Generate the new function
        let signature = self.generate_signature(lang.name(), &params, None);
        let body = self.wrap_body(lang.name(), selected, &indent);
        let new_function = format!("{}{}\n\n", signature, body);

        // Generate the function call
        let call_args = params
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<Vec<_>>()
            .join(", ");
        let call = format!("{}({})", self.name, call_args);

        // Create edit to insert new function before the current function/block
        let insert_pos = Position {
            line: 0,
            character: 0,
        };
        preview.add_edit(TextEdit::insert(
            ctx.target_file.clone(),
            insert_pos,
            new_function,
        ));

        // Generate diff preview (before moving call)
        let diff = format!(
            "Extract to:\n{}{}\n\nReplace with:\n{}",
            signature, body, call
        );

        // Create edit to replace selection with function call
        preview.add_edit(TextEdit::new(
            ctx.target_file.clone(),
            ctx.target_range,
            call,
        ));
        preview = preview.with_diff(diff);

        Ok(preview)
    }

    fn apply(&self, ctx: &mut RefactoringContext) -> Result<RefactoringResult> {
        let preview = self.preview(ctx)?;

        // Apply edits in reverse order (to preserve positions)
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

        // Write the file
        std::fs::write(&ctx.target_file, &new_source)?;
        ctx.source = new_source;

        Ok(
            RefactoringResult::success(format!("Extracted function '{}'", self.name))
                .with_file(ctx.target_file.clone())
                .with_edits(edits),
        )
    }
}

/// Extract a selection into a new variable.
#[derive(Debug, Clone)]
pub struct ExtractVariable {
    /// Name for the new variable.
    pub name: String,
    /// Whether to replace all occurrences.
    pub replace_all: bool,
    /// Whether to make the variable constant.
    pub is_const: bool,
}

impl ExtractVariable {
    /// Create a new ExtractVariable operation.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            replace_all: false,
            is_const: false,
        }
    }

    /// Replace all occurrences of the expression.
    pub fn replace_all_occurrences(mut self) -> Self {
        self.replace_all = true;
        self
    }

    /// Make the variable a constant.
    pub fn as_const(mut self) -> Self {
        self.is_const = true;
        self
    }

    /// Find all occurrences of the selected expression.
    fn find_occurrences(&self, source: &str, expression: &str) -> Vec<Range> {
        let mut occurrences = Vec::new();
        let mut search_start = 0;

        while let Some(pos) = source[search_start..].find(expression) {
            let abs_pos = search_start + pos;

            // Convert to line/column
            let mut line = 0;
            let mut col = 0;
            for (i, ch) in source.char_indices() {
                if i == abs_pos {
                    break;
                }
                if ch == '\n' {
                    line += 1;
                    col = 0;
                } else {
                    col += 1;
                }
            }

            let start = Position {
                line,
                character: col,
            };

            // Find end position
            let _end_offset = abs_pos + expression.len();
            let mut end_line = line;
            let mut end_col = col;
            for ch in expression.chars() {
                if ch == '\n' {
                    end_line += 1;
                    end_col = 0;
                } else {
                    end_col += 1;
                }
            }

            let end = Position {
                line: end_line,
                character: end_col,
            };

            occurrences.push(Range { start, end });
            search_start = abs_pos + 1;
        }

        occurrences
    }

    /// Generate variable declaration for a language.
    fn generate_declaration(&self, lang_name: &str, expression: &str) -> String {
        let keyword = match (lang_name, self.is_const) {
            ("rust", true) => "const",
            ("rust", false) => "let",
            ("typescript" | "javascript", true) => "const",
            ("typescript" | "javascript", false) => "let",
            ("python", _) => "",
            ("go", true) => "const",
            ("go", false) => "",
            ("java", true) => "final var",
            ("java", false) => "var",
            ("csharp", true) => "const var",
            ("csharp", false) => "var",
            ("ruby", _) => "",
            _ => "let",
        };

        match lang_name {
            "rust" => format!("{} {} = {};", keyword, self.name, expression),
            "typescript" | "javascript" => {
                format!("{} {} = {};", keyword, self.name, expression)
            }
            "python" => format!("{} = {}", self.name, expression),
            "go" => {
                if self.is_const {
                    format!("const {} = {}", self.name, expression)
                } else {
                    format!("{} := {}", self.name, expression)
                }
            }
            "java" => format!("{} {} = {};", keyword, self.name, expression),
            "csharp" => format!("{} {} = {};", keyword, self.name, expression),
            "ruby" => format!("{} = {}", self.name, expression),
            _ => format!("{} {} = {};", keyword, self.name, expression),
        }
    }
}

impl RefactoringOperation for ExtractVariable {
    fn name(&self) -> &'static str {
        "Extract Variable"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let selected = ctx.selected_text();
        if selected.trim().is_empty() {
            return Ok(ValidationResult::invalid("No expression selected"));
        }

        if self.name.is_empty() {
            return Ok(ValidationResult::invalid("Variable name is required"));
        }

        // Check for valid identifier
        if !self
            .name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        {
            return Ok(ValidationResult::invalid(
                "Variable name must start with a letter or underscore",
            ));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let mut preview =
            RefactoringPreview::new(format!("Extract expression into variable '{}'", self.name));

        let lang = ctx.language().ok_or_else(|| {
            crate::error::RefactorError::InvalidConfig("No language detected".to_string())
        })?;

        let selected = ctx.selected_text().trim();
        let declaration = self.generate_declaration(lang.name(), selected);
        let indent = ctx.get_indentation(ctx.target_range.start.line);

        // Find occurrences to replace
        let occurrences = if self.replace_all {
            self.find_occurrences(&ctx.source, selected)
        } else {
            vec![ctx.target_range]
        };

        // Insert declaration at the start of the current line
        let insert_line = ctx.target_range.start.line;
        let insert_pos = Position {
            line: insert_line,
            character: 0,
        };
        preview.add_edit(TextEdit::insert(
            ctx.target_file.clone(),
            insert_pos,
            format!("{}{}\n", indent, declaration),
        ));

        // Replace occurrences with variable name
        for range in occurrences {
            preview.add_edit(TextEdit::new(
                ctx.target_file.clone(),
                range,
                self.name.clone(),
            ));
        }

        let diff = format!(
            "Add declaration:\n{}{}\n\nReplace '{}' with '{}'",
            indent, declaration, selected, self.name
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

        Ok(
            RefactoringResult::success(format!("Extracted variable '{}'", self.name))
                .with_file(ctx.target_file.clone())
                .with_edits(edits),
        )
    }
}

/// Extract a selection into a constant.
#[derive(Debug, Clone)]
pub struct ExtractConstant {
    /// Name for the new constant.
    pub name: String,
    /// Visibility of the constant.
    pub visibility: Visibility,
    /// Whether to move to module level.
    pub module_level: bool,
}

impl ExtractConstant {
    /// Create a new ExtractConstant operation.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            visibility: Visibility::Private,
            module_level: true,
        }
    }

    /// Make the constant public.
    pub fn public(mut self) -> Self {
        self.visibility = Visibility::Public;
        self
    }

    /// Keep the constant at local scope.
    pub fn local(mut self) -> Self {
        self.module_level = false;
        self
    }

    /// Generate constant declaration for a language.
    fn generate_declaration(&self, lang_name: &str, expression: &str) -> String {
        let vis = self.visibility.keyword(lang_name);
        let name = if lang_name == "rust" || lang_name == "go" {
            self.name.to_uppercase()
        } else {
            self.name.clone()
        };

        match lang_name {
            "rust" => format!("{}const {}: _ = {};", vis, name, expression),
            "typescript" | "javascript" => format!("{}const {} = {};", vis, name, expression),
            "python" => format!("{} = {}", name, expression),
            "go" => format!("const {} = {}", name, expression),
            "java" => format!("{}static final var {} = {};", vis, name, expression),
            "csharp" => format!("{}const var {} = {};", vis, name, expression),
            "ruby" => format!("{} = {}", name.to_uppercase(), expression),
            _ => format!("const {} = {};", name, expression),
        }
    }
}

impl RefactoringOperation for ExtractConstant {
    fn name(&self) -> &'static str {
        "Extract Constant"
    }

    fn validate(&self, ctx: &RefactoringContext) -> Result<ValidationResult> {
        ctx.validate()?;

        let selected = ctx.selected_text();
        if selected.trim().is_empty() {
            return Ok(ValidationResult::invalid("No expression selected"));
        }

        if self.name.is_empty() {
            return Ok(ValidationResult::invalid("Constant name is required"));
        }

        Ok(ValidationResult::valid())
    }

    fn preview(&self, ctx: &RefactoringContext) -> Result<RefactoringPreview> {
        let mut preview =
            RefactoringPreview::new(format!("Extract expression into constant '{}'", self.name));

        let lang = ctx.language().ok_or_else(|| {
            crate::error::RefactorError::InvalidConfig("No language detected".to_string())
        })?;

        let selected = ctx.selected_text().trim();
        let declaration = self.generate_declaration(lang.name(), selected);

        // Insert at the top of the file (module level)
        let insert_pos = if self.module_level {
            Position {
                line: 0,
                character: 0,
            }
        } else {
            Position {
                line: ctx.target_range.start.line,
                character: 0,
            }
        };

        let indent = if self.module_level {
            String::new()
        } else {
            ctx.get_indentation(ctx.target_range.start.line)
        };

        preview.add_edit(TextEdit::insert(
            ctx.target_file.clone(),
            insert_pos,
            format!("{}{}\n", indent, declaration),
        ));

        // Replace the selected expression with the constant name
        let const_name = if lang.name() == "rust" || lang.name() == "go" || lang.name() == "ruby" {
            self.name.to_uppercase()
        } else {
            self.name.clone()
        };

        preview.add_edit(TextEdit::new(
            ctx.target_file.clone(),
            ctx.target_range,
            const_name.clone(),
        ));

        let diff = format!(
            "Add constant:\n{}\n\nReplace '{}' with '{}'",
            declaration, selected, const_name
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

        Ok(
            RefactoringResult::success(format!("Extracted constant '{}'", self.name))
                .with_file(ctx.target_file.clone())
                .with_edits(edits),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_function_validation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let x = 1 + 2;\n}")
            .with_selection(1, 12, 1, 17);

        let op = ExtractFunction::new("add_numbers");
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_extract_function_empty_name() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    let x = 1 + 2;\n}")
            .with_selection(1, 12, 1, 17);

        let op = ExtractFunction::new("");
        let result = op.validate(&ctx).unwrap();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_extract_variable_validation() {
        let ctx = RefactoringContext::new("/workspace", "test.rs")
            .with_source("fn main() {\n    println!(\"{}\", 1 + 2);\n}")
            .with_selection(1, 19, 1, 24);

        let op = ExtractVariable::new("sum");
        let result = op.validate(&ctx).unwrap();
        assert!(result.is_valid);
    }

    #[test]
    fn test_generate_rust_signature() {
        let op = ExtractFunction::new("my_func").public();
        let params = vec![
            ("x".to_string(), Some("i32".to_string())),
            ("y".to_string(), None),
        ];

        let sig = op.generate_signature("rust", &params, Some("i32"));
        assert!(sig.contains("pub "));
        assert!(sig.contains("fn my_func"));
        assert!(sig.contains("x: i32"));
        assert!(sig.contains("-> i32"));
    }

    #[test]
    fn test_generate_python_declaration() {
        let op = ExtractVariable::new("result");
        let decl = op.generate_declaration("python", "1 + 2");
        assert_eq!(decl, "result = 1 + 2");
    }

    #[test]
    fn test_generate_typescript_declaration() {
        let op = ExtractVariable::new("result").as_const();
        let decl = op.generate_declaration("typescript", "1 + 2");
        assert_eq!(decl, "const result = 1 + 2;");
    }

    #[test]
    fn test_visibility_keywords() {
        assert_eq!(Visibility::Public.keyword("rust"), "pub ");
        assert_eq!(Visibility::Private.keyword("rust"), "");
        assert_eq!(Visibility::Public.keyword("typescript"), "export ");
        assert_eq!(Visibility::Public.keyword("java"), "public ");
        assert_eq!(Visibility::Private.keyword("python"), "_");
    }
}
