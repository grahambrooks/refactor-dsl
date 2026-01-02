//! Git diff reading and API extraction from source files.

use crate::error::{RefactorError, Result};
use crate::lang::{Language, LanguageRegistry};
use git2::{DiffDelta, DiffOptions, Repository};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

use super::change::ApiType;
use super::signature::{ApiSignature, Parameter, SourceLocation, TypeInfo, Visibility};

/// Represents the content of a file at a specific git ref.
#[derive(Debug, Clone)]
pub struct FileContent {
    /// Path relative to repository root.
    pub path: PathBuf,
    /// File content as string.
    pub content: String,
}

/// Change type for a file between two refs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileChangeType {
    Added,
    Deleted,
    Modified,
    Renamed,
}

/// A file change between two git refs.
#[derive(Debug, Clone)]
pub struct FileChange {
    /// Type of change.
    pub change_type: FileChangeType,
    /// Old path (if applicable).
    pub old_path: Option<PathBuf>,
    /// New path (if applicable).
    pub new_path: Option<PathBuf>,
    /// Old content (if applicable).
    pub old_content: Option<String>,
    /// New content (if applicable).
    pub new_content: Option<String>,
}

impl FileChange {
    /// Get the primary path for this change.
    pub fn path(&self) -> &Path {
        self.new_path
            .as_ref()
            .or(self.old_path.as_ref())
            .map(|p| p.as_path())
            .unwrap_or(Path::new(""))
    }
}

/// Reads file differences between two git refs.
pub struct GitDiffReader<'a> {
    repo: &'a Repository,
    language_filter: Option<Vec<String>>,
}

impl<'a> GitDiffReader<'a> {
    /// Create a new diff reader for the given repository.
    pub fn new(repo: &'a Repository) -> Self {
        Self {
            repo,
            language_filter: None,
        }
    }

    /// Filter files by language extension.
    pub fn filter_extensions(mut self, extensions: Vec<String>) -> Self {
        self.language_filter = Some(extensions);
        self
    }

    /// Get the list of changed files between two refs.
    pub fn changed_files(&self, from_ref: &str, to_ref: &str) -> Result<Vec<FileChange>> {
        let from_tree = self.get_tree(from_ref)?;
        let to_tree = self.get_tree(to_ref)?;

        let mut opts = DiffOptions::new();
        opts.ignore_whitespace(true);

        let diff = self
            .repo
            .diff_tree_to_tree(Some(&from_tree), Some(&to_tree), Some(&mut opts))?;

        let mut changes = Vec::new();

        diff.foreach(
            &mut |delta, _| {
                if let Some(change) = self.process_delta(&delta, &from_tree, &to_tree) {
                    // Apply language filter
                    if let Some(ref extensions) = self.language_filter {
                        let path = change.path();
                        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                            if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                                return true; // Skip this file
                            }
                        } else {
                            return true; // Skip files without extensions
                        }
                    }
                    changes.push(change);
                }
                true
            },
            None,
            None,
            None,
        )?;

        Ok(changes)
    }

    /// Get all files at a specific ref.
    pub fn files_at_ref(&self, ref_name: &str) -> Result<Vec<FileContent>> {
        let tree = self.get_tree(ref_name)?;
        let mut files = Vec::new();

        tree.walk(git2::TreeWalkMode::PreOrder, |dir, entry| {
            if entry.kind() == Some(git2::ObjectType::Blob) {
                let path = if dir.is_empty() {
                    PathBuf::from(entry.name().unwrap_or(""))
                } else {
                    PathBuf::from(format!("{}/{}", dir, entry.name().unwrap_or("")))
                };

                // Apply language filter
                if let Some(ref extensions) = self.language_filter {
                    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                        if !extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                            return git2::TreeWalkResult::Ok;
                        }
                    } else {
                        return git2::TreeWalkResult::Ok;
                    }
                }

                if let Ok(blob) = self.repo.find_blob(entry.id()) {
                    if let Ok(content) = std::str::from_utf8(blob.content()) {
                        files.push(FileContent {
                            path,
                            content: content.to_string(),
                        });
                    }
                }
            }
            git2::TreeWalkResult::Ok
        })?;

        Ok(files)
    }

    fn get_tree(&self, ref_name: &str) -> Result<git2::Tree<'a>> {
        let obj = self.repo.revparse_single(ref_name).map_err(|e| {
            RefactorError::InvalidConfig(format!("Invalid git ref '{}': {}", ref_name, e))
        })?;

        // Handle the object ownership properly
        let tree_id = obj.peel_to_tree().map_err(|e| {
            RefactorError::InvalidConfig(format!("Could not get tree for '{}': {}", ref_name, e))
        })?.id();

        self.repo.find_tree(tree_id).map_err(|e| {
            RefactorError::InvalidConfig(format!("Could not find tree for '{}': {}", ref_name, e))
        })
    }

    fn get_file_content(&self, tree: &git2::Tree, path: &Path) -> Option<String> {
        let entry = tree.get_path(path).ok()?;
        let blob = self.repo.find_blob(entry.id()).ok()?;
        std::str::from_utf8(blob.content()).ok().map(String::from)
    }

    fn process_delta(
        &self,
        delta: &DiffDelta,
        from_tree: &git2::Tree,
        to_tree: &git2::Tree,
    ) -> Option<FileChange> {
        use git2::Delta;

        let old_file = delta.old_file();
        let new_file = delta.new_file();

        let old_path = old_file.path().map(PathBuf::from);
        let new_path = new_file.path().map(PathBuf::from);

        let change_type = match delta.status() {
            Delta::Added => FileChangeType::Added,
            Delta::Deleted => FileChangeType::Deleted,
            Delta::Modified => FileChangeType::Modified,
            Delta::Renamed => FileChangeType::Renamed,
            _ => return None,
        };

        let old_content = old_path
            .as_ref()
            .and_then(|p| self.get_file_content(from_tree, p));
        let new_content = new_path
            .as_ref()
            .and_then(|p| self.get_file_content(to_tree, p));

        Some(FileChange {
            change_type,
            old_path,
            new_path,
            old_content,
            new_content,
        })
    }
}

/// Extracts API signatures from source code using tree-sitter.
pub struct ApiExtractor {
    registry: LanguageRegistry,
}

impl Default for ApiExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl ApiExtractor {
    /// Create a new API extractor with all supported languages.
    pub fn new() -> Self {
        Self {
            registry: LanguageRegistry::new(),
        }
    }

    /// Create an API extractor with a custom language registry.
    pub fn with_registry(registry: LanguageRegistry) -> Self {
        Self { registry }
    }

    /// Extract API signatures from a source file.
    pub fn extract(&self, path: &Path, source: &str) -> Result<Vec<ApiSignature>> {
        let lang = self.registry.detect(path).ok_or_else(|| {
            RefactorError::UnsupportedLanguage(
                path.extension()
                    .and_then(|e| e.to_str())
                    .unwrap_or("unknown")
                    .to_string(),
            )
        })?;

        self.extract_with_language(path, source, lang)
    }

    /// Extract API signatures using a specific language.
    pub fn extract_with_language(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
    ) -> Result<Vec<ApiSignature>> {
        match lang.name() {
            "rust" => self.extract_rust(path, source, lang),
            "typescript" => self.extract_typescript(path, source, lang),
            "python" => self.extract_python(path, source, lang),
            _ => Ok(Vec::new()),
        }
    }

    /// Extract from multiple files.
    pub fn extract_all(&self, files: &[FileContent]) -> Result<HashMap<PathBuf, Vec<ApiSignature>>> {
        let mut result = HashMap::new();

        for file in files {
            match self.extract(&file.path, &file.content) {
                Ok(signatures) => {
                    if !signatures.is_empty() {
                        result.insert(file.path.clone(), signatures);
                    }
                }
                Err(e) => {
                    // Log but continue - some files might not be parseable
                    eprintln!("Warning: Failed to extract from {:?}: {}", file.path, e);
                }
            }
        }

        Ok(result)
    }

    fn extract_rust(&self, path: &Path, source: &str, lang: &dyn Language) -> Result<Vec<ApiSignature>> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();
        let mut signatures = Vec::new();

        // Query for public functions
        let fn_query = lang.query(
            r#"
            (function_item
                (visibility_modifier)? @vis
                name: (identifier) @fn_name
                parameters: (parameters) @params
                return_type: (_)? @return_type
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut is_pub = false;
            let mut params_node = None;
            let mut return_node = None;
            let mut fn_node = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "vis" => {
                        let vis_text = capture.node.utf8_text(source_bytes).unwrap_or("");
                        is_pub = vis_text.starts_with("pub");
                    }
                    "params" => {
                        params_node = Some(capture.node);
                    }
                    "return_type" => {
                        return_node = Some(capture.node);
                    }
                    "function" => {
                        fn_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(fn_n)) = (fn_name, fn_node) {
                let location = SourceLocation::new(
                    path,
                    fn_n.start_position().row + 1,
                    fn_n.start_position().column + 1,
                );

                let visibility = if is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let parameters = params_node
                    .map(|n| self.parse_rust_params(n, source_bytes))
                    .unwrap_or_default();

                let return_type = return_node
                    .and_then(|n| n.utf8_text(source_bytes).ok())
                    .map(|t| TypeInfo::simple(t.trim()));

                let sig = ApiSignature::function(name, location)
                    .with_visibility(visibility)
                    .with_params(parameters)
                    .exported(is_pub);

                let sig = if let Some(rt) = return_type {
                    sig.with_return_type(rt)
                } else {
                    sig
                };

                signatures.push(sig);
            }
        }

        // Query for structs
        let struct_query = lang.query(
            r#"
            (struct_item
                (visibility_modifier)? @vis
                name: (type_identifier) @struct_name
            ) @struct
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&struct_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut struct_name = None;
            let mut is_pub = false;
            let mut struct_node = None;

            for capture in m.captures {
                let name = struct_query.capture_names()[capture.index as usize];
                match name {
                    "struct_name" => {
                        struct_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "vis" => {
                        let vis_text = capture.node.utf8_text(source_bytes).unwrap_or("");
                        is_pub = vis_text.starts_with("pub");
                    }
                    "struct" => {
                        struct_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(s_node)) = (struct_name, struct_node) {
                let location = SourceLocation::new(
                    path,
                    s_node.start_position().row + 1,
                    s_node.start_position().column + 1,
                );

                let visibility = if is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let sig = ApiSignature::type_def(name, ApiType::Struct, location)
                    .with_visibility(visibility)
                    .exported(is_pub);

                signatures.push(sig);
            }
        }

        // Query for enums
        let enum_query = lang.query(
            r#"
            (enum_item
                (visibility_modifier)? @vis
                name: (type_identifier) @enum_name
            ) @enum
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&enum_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut enum_name = None;
            let mut is_pub = false;
            let mut enum_node = None;

            for capture in m.captures {
                let name = enum_query.capture_names()[capture.index as usize];
                match name {
                    "enum_name" => {
                        enum_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "vis" => {
                        let vis_text = capture.node.utf8_text(source_bytes).unwrap_or("");
                        is_pub = vis_text.starts_with("pub");
                    }
                    "enum" => {
                        enum_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(e_node)) = (enum_name, enum_node) {
                let location = SourceLocation::new(
                    path,
                    e_node.start_position().row + 1,
                    e_node.start_position().column + 1,
                );

                let visibility = if is_pub {
                    Visibility::Public
                } else {
                    Visibility::Private
                };

                let sig = ApiSignature::type_def(name, ApiType::Enum, location)
                    .with_visibility(visibility)
                    .exported(is_pub);

                signatures.push(sig);
            }
        }

        Ok(signatures)
    }

    fn parse_rust_params(&self, params_node: tree_sitter::Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();

        for i in 0..params_node.child_count() {
            if let Some(child) = params_node.child(i as u32) {
                if child.kind() == "parameter" {
                    let mut param_name = None;
                    let mut param_type = None;

                    for j in 0..child.child_count() {
                        if let Some(sub) = child.child(j as u32) {
                            match sub.kind() {
                                "identifier" => {
                                    param_name = sub.utf8_text(source).ok().map(String::from);
                                }
                                _ if sub.kind().contains("type") || sub.kind() == "_type" => {
                                    param_type = sub.utf8_text(source).ok().map(|t| TypeInfo::simple(t.trim()));
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(name) = param_name {
                        let mut param = Parameter::new(name);
                        if let Some(ty) = param_type {
                            param = param.with_type(ty);
                        }
                        params.push(param);
                    }
                }
            }
        }

        params
    }

    fn extract_typescript(&self, path: &Path, source: &str, lang: &dyn Language) -> Result<Vec<ApiSignature>> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();
        let mut signatures = Vec::new();

        // Query for exported functions
        let fn_query = lang.query(
            r#"
            (export_statement
                declaration: (function_declaration
                    name: (identifier) @fn_name
                    parameters: (formal_parameters) @params
                    return_type: (type_annotation)? @return_type
                )
            ) @export_fn
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut params_node = None;
            let mut return_node = None;
            let mut fn_node = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "params" => {
                        params_node = Some(capture.node);
                    }
                    "return_type" => {
                        return_node = Some(capture.node);
                    }
                    "export_fn" => {
                        fn_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(fn_n)) = (fn_name, fn_node) {
                let location = SourceLocation::new(
                    path,
                    fn_n.start_position().row + 1,
                    fn_n.start_position().column + 1,
                );

                let parameters = params_node
                    .map(|n| self.parse_ts_params(n, source_bytes))
                    .unwrap_or_default();

                let return_type = return_node
                    .and_then(|n| n.utf8_text(source_bytes).ok())
                    .map(|t| {
                        let t = t.trim().trim_start_matches(':').trim();
                        TypeInfo::simple(t)
                    });

                let mut sig = ApiSignature::function(name, location)
                    .with_visibility(Visibility::Public)
                    .with_params(parameters)
                    .exported(true);

                if let Some(rt) = return_type {
                    sig = sig.with_return_type(rt);
                }

                signatures.push(sig);
            }
        }

        // Query for regular function declarations (non-exported)
        let fn_query2 = lang.query(
            r#"
            (function_declaration
                name: (identifier) @fn_name
                parameters: (formal_parameters) @params
                return_type: (type_annotation)? @return_type
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query2, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut params_node = None;
            let mut return_node = None;
            let mut fn_node = None;

            for capture in m.captures {
                let name = fn_query2.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "params" => {
                        params_node = Some(capture.node);
                    }
                    "return_type" => {
                        return_node = Some(capture.node);
                    }
                    "function" => {
                        fn_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(fn_n)) = (fn_name, fn_node) {
                // Skip if already captured as exported
                if signatures.iter().any(|s| s.name == name) {
                    continue;
                }

                let location = SourceLocation::new(
                    path,
                    fn_n.start_position().row + 1,
                    fn_n.start_position().column + 1,
                );

                let parameters = params_node
                    .map(|n| self.parse_ts_params(n, source_bytes))
                    .unwrap_or_default();

                let return_type = return_node
                    .and_then(|n| n.utf8_text(source_bytes).ok())
                    .map(|t| {
                        let t = t.trim().trim_start_matches(':').trim();
                        TypeInfo::simple(t)
                    });

                let mut sig = ApiSignature::function(name, location)
                    .with_visibility(Visibility::Private)
                    .with_params(parameters)
                    .exported(false);

                if let Some(rt) = return_type {
                    sig = sig.with_return_type(rt);
                }

                signatures.push(sig);
            }
        }

        // Query for exported interfaces
        let iface_query = lang.query(
            r#"
            (export_statement
                declaration: (interface_declaration
                    name: (type_identifier) @iface_name
                )
            ) @export_iface
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&iface_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut iface_name = None;
            let mut iface_node = None;

            for capture in m.captures {
                let name = iface_query.capture_names()[capture.index as usize];
                match name {
                    "iface_name" => {
                        iface_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "export_iface" => {
                        iface_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(i_node)) = (iface_name, iface_node) {
                let location = SourceLocation::new(
                    path,
                    i_node.start_position().row + 1,
                    i_node.start_position().column + 1,
                );

                let sig = ApiSignature::type_def(name, ApiType::Interface, location)
                    .with_visibility(Visibility::Public)
                    .exported(true);

                signatures.push(sig);
            }
        }

        // Query for exported classes
        let class_query = lang.query(
            r#"
            (export_statement
                declaration: (class_declaration
                    name: (type_identifier) @class_name
                )
            ) @export_class
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut class_name = None;
            let mut class_node = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "export_class" => {
                        class_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(c_node)) = (class_name, class_node) {
                let location = SourceLocation::new(
                    path,
                    c_node.start_position().row + 1,
                    c_node.start_position().column + 1,
                );

                let sig = ApiSignature::type_def(name, ApiType::Class, location)
                    .with_visibility(Visibility::Public)
                    .exported(true);

                signatures.push(sig);
            }
        }

        Ok(signatures)
    }

    fn parse_ts_params(&self, params_node: tree_sitter::Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();

        for i in 0..params_node.child_count() {
            if let Some(child) = params_node.child(i as u32) {
                let kind = child.kind();
                if kind == "required_parameter" || kind == "optional_parameter" {
                    let mut param_name = None;
                    let mut param_type = None;
                    let is_optional = kind == "optional_parameter";

                    for j in 0..child.child_count() {
                        if let Some(sub) = child.child(j as u32) {
                            match sub.kind() {
                                "identifier" => {
                                    param_name = sub.utf8_text(source).ok().map(String::from);
                                }
                                "type_annotation" => {
                                    let type_text = sub.utf8_text(source).ok();
                                    param_type = type_text.map(|t| {
                                        TypeInfo::simple(t.trim().trim_start_matches(':').trim())
                                    });
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(name) = param_name {
                        let mut param = Parameter::new(name);
                        if let Some(ty) = param_type {
                            param = param.with_type(ty);
                        }
                        if is_optional {
                            param = param.optional();
                        }
                        params.push(param);
                    }
                }
            }
        }

        params
    }

    fn extract_python(&self, path: &Path, source: &str, lang: &dyn Language) -> Result<Vec<ApiSignature>> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();
        let mut signatures = Vec::new();

        // Query for function definitions
        let fn_query = lang.query(
            r#"
            (function_definition
                name: (identifier) @fn_name
                parameters: (parameters) @params
                return_type: (type)? @return_type
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut params_node = None;
            let mut return_node = None;
            let mut fn_node = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "params" => {
                        params_node = Some(capture.node);
                    }
                    "return_type" => {
                        return_node = Some(capture.node);
                    }
                    "function" => {
                        fn_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(fn_n)) = (fn_name, fn_node) {
                // In Python, functions starting with _ are considered private
                let is_private = name.starts_with('_');
                let visibility = if is_private {
                    Visibility::Private
                } else {
                    Visibility::Public
                };

                let location = SourceLocation::new(
                    path,
                    fn_n.start_position().row + 1,
                    fn_n.start_position().column + 1,
                );

                let parameters = params_node
                    .map(|n| self.parse_python_params(n, source_bytes))
                    .unwrap_or_default();

                let return_type = return_node
                    .and_then(|n| n.utf8_text(source_bytes).ok())
                    .map(|t| TypeInfo::simple(t.trim()));

                let mut sig = ApiSignature::function(name, location)
                    .with_visibility(visibility)
                    .with_params(parameters)
                    .exported(!is_private);

                if let Some(rt) = return_type {
                    sig = sig.with_return_type(rt);
                }

                signatures.push(sig);
            }
        }

        // Query for class definitions
        let class_query = lang.query(
            r#"
            (class_definition
                name: (identifier) @class_name
            ) @class
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut class_name = None;
            let mut class_node = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "class" => {
                        class_node = Some(capture.node);
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(c_node)) = (class_name, class_node) {
                let is_private = name.starts_with('_');
                let visibility = if is_private {
                    Visibility::Private
                } else {
                    Visibility::Public
                };

                let location = SourceLocation::new(
                    path,
                    c_node.start_position().row + 1,
                    c_node.start_position().column + 1,
                );

                let sig = ApiSignature::type_def(name, ApiType::Class, location)
                    .with_visibility(visibility)
                    .exported(!is_private);

                signatures.push(sig);
            }
        }

        Ok(signatures)
    }

    fn parse_python_params(&self, params_node: tree_sitter::Node, source: &[u8]) -> Vec<Parameter> {
        let mut params = Vec::new();

        for i in 0..params_node.child_count() {
            if let Some(child) = params_node.child(i as u32) {
                let kind = child.kind();
                if kind == "identifier" {
                    // Simple parameter
                    if let Ok(name) = child.utf8_text(source) {
                        // Skip 'self' parameter
                        if name != "self" && name != "cls" {
                            params.push(Parameter::new(name));
                        }
                    }
                } else if kind == "typed_parameter" || kind == "default_parameter" || kind == "typed_default_parameter" {
                    let mut param_name = None;
                    let mut param_type = None;
                    let has_default = kind.contains("default");

                    for j in 0..child.child_count() {
                        if let Some(sub) = child.child(j as u32) {
                            match sub.kind() {
                                "identifier" => {
                                    param_name = sub.utf8_text(source).ok().map(String::from);
                                }
                                "type" => {
                                    param_type = sub.utf8_text(source).ok().map(|t| TypeInfo::simple(t.trim()));
                                }
                                _ => {}
                            }
                        }
                    }

                    if let Some(name) = param_name {
                        if name != "self" && name != "cls" {
                            let mut param = Parameter::new(name);
                            if let Some(ty) = param_type {
                                param = param.with_type(ty);
                            }
                            if has_default {
                                param = param.with_default();
                            }
                            params.push(param);
                        }
                    }
                } else if kind == "list_splat_pattern" || kind == "dictionary_splat_pattern" {
                    // *args or **kwargs
                    if let Some(ident) = child.child_by_field_name("name") {
                        if let Ok(name) = ident.utf8_text(source) {
                            let param = Parameter::new(name).variadic();
                            params.push(param);
                        }
                    }
                }
            }
        }

        params
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_extract_rust_functions() {
        let extractor = ApiExtractor::new();
        let source = r#"
pub fn hello(name: &str) -> String {
    format!("Hello, {}", name)
}

fn private_fn() {}

pub fn greet(name: &str, times: u32) {
    for _ in 0..times {
        println!("Hi, {}", name);
    }
}
"#;

        let sigs = extractor.extract(Path::new("test.rs"), source).unwrap();

        assert_eq!(sigs.len(), 3);

        let hello = sigs.iter().find(|s| s.name == "hello").unwrap();
        assert!(hello.is_exported);
        assert_eq!(hello.parameters.len(), 1);
        assert_eq!(hello.parameters[0].name, "name");

        let private_fn = sigs.iter().find(|s| s.name == "private_fn").unwrap();
        assert!(!private_fn.is_exported);
    }

    #[test]
    fn test_extract_rust_structs() {
        let extractor = ApiExtractor::new();
        let source = r#"
pub struct Point {
    x: i32,
    y: i32,
}

struct InternalData {
    value: String,
}

pub enum Status {
    Active,
    Inactive,
}
"#;

        let sigs = extractor.extract(Path::new("test.rs"), source).unwrap();

        assert_eq!(sigs.len(), 3);

        let point = sigs.iter().find(|s| s.name == "Point").unwrap();
        assert!(point.is_exported);
        assert_eq!(point.kind, ApiType::Struct);

        let internal = sigs.iter().find(|s| s.name == "InternalData").unwrap();
        assert!(!internal.is_exported);

        let status = sigs.iter().find(|s| s.name == "Status").unwrap();
        assert!(status.is_exported);
        assert_eq!(status.kind, ApiType::Enum);
    }

    #[test]
    fn test_extract_typescript_functions() {
        let extractor = ApiExtractor::new();
        let source = r#"
export function greet(name: string): string {
    return `Hello, ${name}`;
}

function privateHelper(): void {
    console.log("internal");
}

export interface User {
    id: number;
    name: string;
}
"#;

        let sigs = extractor.extract(Path::new("test.ts"), source).unwrap();

        let greet = sigs.iter().find(|s| s.name == "greet").unwrap();
        assert!(greet.is_exported);
        assert_eq!(greet.kind, ApiType::Function);

        let user = sigs.iter().find(|s| s.name == "User").unwrap();
        assert!(user.is_exported);
        assert_eq!(user.kind, ApiType::Interface);
    }

    #[test]
    fn test_extract_python_functions() {
        let extractor = ApiExtractor::new();
        let source = r#"
def greet(name: str) -> str:
    return f"Hello, {name}"

def _private_helper():
    pass

class UserService:
    def get_user(self, id: int):
        pass
"#;

        let sigs = extractor.extract(Path::new("test.py"), source).unwrap();

        let greet = sigs.iter().find(|s| s.name == "greet").unwrap();
        assert!(greet.is_exported);
        assert_eq!(greet.parameters.len(), 1);
        assert_eq!(greet.parameters[0].name, "name");

        let private_helper = sigs.iter().find(|s| s.name == "_private_helper").unwrap();
        assert!(!private_helper.is_exported);

        let user_service = sigs.iter().find(|s| s.name == "UserService").unwrap();
        assert!(user_service.is_exported);
        assert_eq!(user_service.kind, ApiType::Class);
    }

    #[test]
    fn test_file_change_path() {
        let change = FileChange {
            change_type: FileChangeType::Modified,
            old_path: Some(PathBuf::from("old.rs")),
            new_path: Some(PathBuf::from("new.rs")),
            old_content: None,
            new_content: None,
        };

        assert_eq!(change.path(), Path::new("new.rs"));

        let deleted = FileChange {
            change_type: FileChangeType::Deleted,
            old_path: Some(PathBuf::from("deleted.rs")),
            new_path: None,
            old_content: None,
            new_content: None,
        };

        assert_eq!(deleted.path(), Path::new("deleted.rs"));
    }
}
