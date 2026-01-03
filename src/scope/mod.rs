//! Scope analysis for semantic refactoring operations.
//!
//! This module provides binding tracking, reference resolution, and usage analysis
//! to enable safe refactoring operations like rename, extract, and delete.
//!
//! ## Overview
//!
//! - **Binding**: A named entity (variable, function, class, etc.)
//! - **Scope**: A region where bindings are visible
//! - **Reference**: A use of a binding in the code
//! - **Usage**: Analysis of how bindings are used
//!
//! ## Example
//!
//! ```rust,no_run
//! use refactor::scope::{ScopeAnalyzer, BindingKind};
//! use std::path::Path;
//!
//! let mut analyzer = ScopeAnalyzer::new();
//!
//! // Analyze a file
//! let source = r#"
//! fn helper() -> i32 { 42 }
//! fn main() { let x = helper(); }
//! "#;
//!
//! analyzer.analyze_file(Path::new("main.rs"), source)?;
//!
//! // Find usages of a function
//! if let Some(binding) = analyzer.find_binding("helper") {
//!     let usages = analyzer.find_usages(&binding);
//!     println!("Found {} usages of 'helper'", usages.len());
//! }
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```

mod binding;
mod reference;
mod usage;

pub use binding::{Binding, BindingKind, BindingTracker, Scope, ScopeId, ScopeKind};
pub use reference::{
    Reference, ReferenceIndex, ReferenceKind, ResolutionConfidence, ResolvedReference,
};
pub use usage::{DeadCodeInfo, SafeDeleteResult, UnusedBinding, UsageAnalyzer, UsageInfo};

use crate::error::Result;
use crate::lang::{Language, LanguageRegistry};
use crate::lsp::{Position, Range};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use streaming_iterator::StreamingIterator;
use tree_sitter::QueryCursor;

/// High-level scope analyzer for multi-file analysis.
#[derive(Default)]
pub struct ScopeAnalyzer {
    /// Per-file binding trackers.
    trackers: HashMap<PathBuf, BindingTracker>,
    /// Cross-file usage analyzer.
    usage_analyzer: UsageAnalyzer,
    /// Language registry.
    registry: LanguageRegistry,
}

impl ScopeAnalyzer {
    /// Create a new scope analyzer.
    pub fn new() -> Self {
        Self {
            trackers: HashMap::new(),
            usage_analyzer: UsageAnalyzer::new(),
            registry: LanguageRegistry::new(),
        }
    }

    /// Create with a custom language registry.
    pub fn with_registry(registry: LanguageRegistry) -> Self {
        Self {
            trackers: HashMap::new(),
            usage_analyzer: UsageAnalyzer::new(),
            registry,
        }
    }

    /// Analyze a source file and extract bindings.
    pub fn analyze_file(&mut self, path: &Path, source: &str) -> Result<()> {
        let lang = match self.registry.detect(path) {
            Some(l) => l,
            None => return Ok(()), // Skip unsupported files
        };

        let mut tracker = BindingTracker::new();
        self.extract_bindings(path, source, lang, &mut tracker)?;

        // Add bindings to the usage analyzer
        for binding in tracker.all_bindings() {
            self.usage_analyzer.add_binding(binding.clone());
        }

        self.trackers.insert(path.to_path_buf(), tracker);
        Ok(())
    }

    /// Extract bindings from source code.
    fn extract_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        match lang.name() {
            "rust" => self.extract_rust_bindings(path, source, lang, tracker),
            "typescript" => self.extract_typescript_bindings(path, source, lang, tracker),
            "python" => self.extract_python_bindings(path, source, lang, tracker),
            "go" => self.extract_go_bindings(path, source, lang, tracker),
            "java" => self.extract_java_bindings(path, source, lang, tracker),
            "csharp" => self.extract_csharp_bindings(path, source, lang, tracker),
            "ruby" => self.extract_ruby_bindings(path, source, lang, tracker),
            _ => Ok(()),
        }
    }

    /// Extract bindings from Rust source.
    fn extract_rust_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract functions
        let fn_query = lang.query(
            r#"
            (function_item
                (visibility_modifier)? @vis
                name: (identifier) @fn_name
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut is_pub = false;
            let mut fn_range = None;

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
                    "function" => {
                        fn_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (fn_name, fn_range) {
                let mut binding =
                    Binding::new(name, BindingKind::Function, path.to_path_buf(), range);
                if is_pub {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract structs
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
            let mut struct_range = None;

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
                        struct_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (struct_name, struct_range) {
                let mut binding =
                    Binding::new(name, BindingKind::Struct, path.to_path_buf(), range);
                if is_pub {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from TypeScript source.
    fn extract_typescript_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract exported functions
        let fn_query = lang.query(
            r#"
            (export_statement
                declaration: (function_declaration
                    name: (identifier) @fn_name
                )
            ) @export_fn
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut fn_range = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "export_fn" => {
                        fn_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (fn_name, fn_range) {
                let binding =
                    Binding::new(name, BindingKind::Function, path.to_path_buf(), range).exported();
                tracker.add_binding(binding);
            }
        }

        // Extract classes
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
            let mut class_range = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "export_class" => {
                        class_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (class_name, class_range) {
                let binding =
                    Binding::new(name, BindingKind::Class, path.to_path_buf(), range).exported();
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from Python source.
    fn extract_python_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract functions
        let fn_query = lang.query(
            r#"
            (function_definition
                name: (identifier) @fn_name
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut fn_range = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "function" => {
                        fn_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (fn_name, fn_range) {
                let is_private = name.starts_with('_');
                let mut binding =
                    Binding::new(name, BindingKind::Function, path.to_path_buf(), range);
                if !is_private {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract classes
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
            let mut class_range = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "class" => {
                        class_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (class_name, class_range) {
                let is_private = name.starts_with('_');
                let mut binding = Binding::new(name, BindingKind::Class, path.to_path_buf(), range);
                if !is_private {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from Go source.
    fn extract_go_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract functions
        let fn_query = lang.query(
            r#"
            (function_declaration
                name: (identifier) @fn_name
            ) @function
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&fn_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut fn_name = None;
            let mut fn_range = None;

            for capture in m.captures {
                let name = fn_query.capture_names()[capture.index as usize];
                match name {
                    "fn_name" => {
                        fn_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "function" => {
                        fn_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (fn_name, fn_range) {
                let is_exported = name.chars().next().is_some_and(|c| c.is_uppercase());
                let mut binding =
                    Binding::new(name, BindingKind::Function, path.to_path_buf(), range);
                if is_exported {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract structs
        let struct_query = lang.query(
            r#"
            (type_declaration
                (type_spec
                    name: (type_identifier) @struct_name
                    type: (struct_type)
                )
            ) @struct
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&struct_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut struct_name = None;
            let mut struct_range = None;

            for capture in m.captures {
                let name = struct_query.capture_names()[capture.index as usize];
                match name {
                    "struct_name" => {
                        struct_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "struct" => {
                        struct_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (struct_name, struct_range) {
                let is_exported = name.chars().next().is_some_and(|c| c.is_uppercase());
                let mut binding =
                    Binding::new(name, BindingKind::Struct, path.to_path_buf(), range);
                if is_exported {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from Java source.
    fn extract_java_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract classes
        let class_query = lang.query(
            r#"
            (class_declaration
                (modifiers)? @modifiers
                name: (identifier) @class_name
            ) @class
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut class_name = None;
            let mut is_public = false;
            let mut class_range = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "modifiers" => {
                        let mods = capture.node.utf8_text(source_bytes).unwrap_or("");
                        is_public = mods.contains("public");
                    }
                    "class" => {
                        class_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (class_name, class_range) {
                let mut binding = Binding::new(name, BindingKind::Class, path.to_path_buf(), range);
                if is_public {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract methods
        let method_query = lang.query(
            r#"
            (method_declaration
                (modifiers)? @modifiers
                name: (identifier) @method_name
            ) @method
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&method_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut method_name = None;
            let mut is_public = false;
            let mut method_range = None;

            for capture in m.captures {
                let name = method_query.capture_names()[capture.index as usize];
                match name {
                    "method_name" => {
                        method_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "modifiers" => {
                        let mods = capture.node.utf8_text(source_bytes).unwrap_or("");
                        is_public = mods.contains("public");
                    }
                    "method" => {
                        method_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (method_name, method_range) {
                let mut binding =
                    Binding::new(name, BindingKind::Method, path.to_path_buf(), range);
                if is_public {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from C# source.
    fn extract_csharp_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract classes
        let class_query = lang.query(
            r#"
            (class_declaration
                (modifier)* @modifiers
                name: (identifier) @class_name
            ) @class
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut class_name = None;
            let mut is_public = false;
            let mut class_range = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "modifiers" => {
                        let mods = capture.node.utf8_text(source_bytes).unwrap_or("");
                        if mods == "public" {
                            is_public = true;
                        }
                    }
                    "class" => {
                        class_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (class_name, class_range) {
                let mut binding = Binding::new(name, BindingKind::Class, path.to_path_buf(), range);
                if is_public {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract methods
        let method_query = lang.query(
            r#"
            (method_declaration
                (modifier)* @modifiers
                name: (identifier) @method_name
            ) @method
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&method_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut method_name = None;
            let mut is_public = false;
            let mut method_range = None;

            for capture in m.captures {
                let name = method_query.capture_names()[capture.index as usize];
                match name {
                    "method_name" => {
                        method_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "modifiers" => {
                        let mods = capture.node.utf8_text(source_bytes).unwrap_or("");
                        if mods == "public" {
                            is_public = true;
                        }
                    }
                    "method" => {
                        method_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (method_name, method_range) {
                let mut binding =
                    Binding::new(name, BindingKind::Method, path.to_path_buf(), range);
                if is_public {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Extract bindings from Ruby source.
    fn extract_ruby_bindings(
        &self,
        path: &Path,
        source: &str,
        lang: &dyn Language,
        tracker: &mut BindingTracker,
    ) -> Result<()> {
        let tree = lang.parse(source)?;
        let source_bytes = source.as_bytes();

        // Extract methods
        let method_query = lang.query(
            r#"
            (method
                name: (identifier) @method_name
            ) @method
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&method_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut method_name = None;
            let mut method_range = None;

            for capture in m.captures {
                let name = method_query.capture_names()[capture.index as usize];
                match name {
                    "method_name" => {
                        method_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "method" => {
                        method_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (method_name, method_range) {
                let is_private = name.starts_with('_');
                let mut binding =
                    Binding::new(name, BindingKind::Method, path.to_path_buf(), range);
                if !is_private {
                    binding = binding.exported();
                }
                tracker.add_binding(binding);
            }
        }

        // Extract classes
        let class_query = lang.query(
            r#"
            (class
                name: (constant) @class_name
            ) @class
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&class_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut class_name = None;
            let mut class_range = None;

            for capture in m.captures {
                let name = class_query.capture_names()[capture.index as usize];
                match name {
                    "class_name" => {
                        class_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "class" => {
                        class_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (class_name, class_range) {
                let binding =
                    Binding::new(name, BindingKind::Class, path.to_path_buf(), range).exported();
                tracker.add_binding(binding);
            }
        }

        // Extract modules
        let module_query = lang.query(
            r#"
            (module
                name: (constant) @module_name
            ) @module
            "#,
        )?;

        let mut cursor = QueryCursor::new();
        let mut matches = cursor.matches(&module_query, tree.root_node(), source_bytes);

        while let Some(m) = matches.next() {
            let mut module_name = None;
            let mut module_range = None;

            for capture in m.captures {
                let name = module_query.capture_names()[capture.index as usize];
                match name {
                    "module_name" => {
                        module_name = capture.node.utf8_text(source_bytes).ok();
                    }
                    "module" => {
                        module_range = Some(node_to_range(&capture.node));
                    }
                    _ => {}
                }
            }

            if let (Some(name), Some(range)) = (module_name, module_range) {
                let binding =
                    Binding::new(name, BindingKind::Module, path.to_path_buf(), range).exported();
                tracker.add_binding(binding);
            }
        }

        Ok(())
    }

    /// Find a binding by name.
    pub fn find_binding(&self, name: &str) -> Option<Binding> {
        for tracker in self.trackers.values() {
            let bindings = tracker.find_by_name(name);
            if let Some(binding) = bindings.first() {
                return Some((*binding).clone());
            }
        }
        None
    }

    /// Find all bindings with a given name.
    pub fn find_all_bindings(&self, name: &str) -> Vec<Binding> {
        let mut result = Vec::new();
        for tracker in self.trackers.values() {
            for binding in tracker.find_by_name(name) {
                result.push(binding.clone());
            }
        }
        result
    }

    /// Find usages of a binding.
    pub fn find_usages(&self, binding: &Binding) -> Vec<&Reference> {
        self.usage_analyzer.find_all_usages(binding)
    }

    /// Analyze usage for a binding.
    pub fn analyze_usage(&mut self, binding: &Binding) -> UsageInfo {
        self.usage_analyzer.analyze_binding(binding)
    }

    /// Find dead code across all analyzed files.
    pub fn find_dead_code(&mut self) -> DeadCodeInfo {
        self.usage_analyzer.find_dead_code()
    }

    /// Check if a binding can be safely deleted.
    pub fn can_safely_delete(&mut self, binding: &Binding) -> SafeDeleteResult {
        self.usage_analyzer.can_safely_delete(binding)
    }

    /// Get the binding tracker for a file.
    pub fn tracker_for_file(&self, path: &Path) -> Option<&BindingTracker> {
        self.trackers.get(path)
    }

    /// Get all analyzed files.
    pub fn analyzed_files(&self) -> impl Iterator<Item = &PathBuf> {
        self.trackers.keys()
    }

    /// Get usage analyzer.
    pub fn usage_analyzer(&self) -> &UsageAnalyzer {
        &self.usage_analyzer
    }

    /// Get mutable usage analyzer.
    pub fn usage_analyzer_mut(&mut self) -> &mut UsageAnalyzer {
        &mut self.usage_analyzer
    }
}

/// Convert a tree-sitter node to an LSP Range.
fn node_to_range(node: &tree_sitter::Node) -> Range {
    Range {
        start: Position {
            line: node.start_position().row as u32,
            character: node.start_position().column as u32,
        },
        end: Position {
            line: node.end_position().row as u32,
            character: node.end_position().column as u32,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_analyzer_rust() {
        let mut analyzer = ScopeAnalyzer::new();
        let source = r#"
pub fn hello() {}
fn private_func() {}
pub struct User {}
"#;

        analyzer.analyze_file(Path::new("test.rs"), source).unwrap();

        let hello = analyzer.find_binding("hello");
        assert!(hello.is_some());
        assert!(hello.unwrap().is_exported);

        let private = analyzer.find_binding("private_func");
        assert!(private.is_some());
        assert!(!private.unwrap().is_exported);

        let user = analyzer.find_binding("User");
        assert!(user.is_some());
        assert_eq!(user.unwrap().kind, BindingKind::Struct);
    }

    #[test]
    fn test_scope_analyzer_go() {
        let mut analyzer = ScopeAnalyzer::new();
        let source = r#"
package main

func HelloWorld() {}
func privateFunc() {}
type User struct {}
"#;

        analyzer.analyze_file(Path::new("main.go"), source).unwrap();

        let hello = analyzer.find_binding("HelloWorld");
        assert!(hello.is_some());
        assert!(hello.unwrap().is_exported);

        let private = analyzer.find_binding("privateFunc");
        assert!(private.is_some());
        assert!(!private.unwrap().is_exported);
    }

    #[test]
    fn test_scope_analyzer_python() {
        let mut analyzer = ScopeAnalyzer::new();
        let source = r#"
def public_func():
    pass

def _private_func():
    pass

class MyClass:
    pass
"#;

        analyzer.analyze_file(Path::new("test.py"), source).unwrap();

        let public = analyzer.find_binding("public_func");
        assert!(public.is_some());
        assert!(public.unwrap().is_exported);

        let private = analyzer.find_binding("_private_func");
        assert!(private.is_some());
        assert!(!private.unwrap().is_exported);
    }

    #[test]
    fn test_scope_analyzer_java() {
        let mut analyzer = ScopeAnalyzer::new();
        let source = r#"
public class UserService {
    public void getUser() {}
    private void helper() {}
}
"#;

        analyzer
            .analyze_file(Path::new("UserService.java"), source)
            .unwrap();

        let service = analyzer.find_binding("UserService");
        assert!(service.is_some());
        assert!(service.unwrap().is_exported);

        let get_user = analyzer.find_binding("getUser");
        assert!(get_user.is_some());
        assert!(get_user.unwrap().is_exported);

        let helper = analyzer.find_binding("helper");
        assert!(helper.is_some());
        assert!(!helper.unwrap().is_exported);
    }

    #[test]
    fn test_scope_analyzer_ruby() {
        let mut analyzer = ScopeAnalyzer::new();
        let source = r#"
module MyModule
  class MyClass
    def public_method
    end

    def _private_method
    end
  end
end
"#;

        analyzer
            .analyze_file(Path::new("my_module.rb"), source)
            .unwrap();

        let module = analyzer.find_binding("MyModule");
        assert!(module.is_some());
        assert_eq!(module.unwrap().kind, BindingKind::Module);

        let class = analyzer.find_binding("MyClass");
        assert!(class.is_some());
        assert_eq!(class.unwrap().kind, BindingKind::Class);

        let public_method = analyzer.find_binding("public_method");
        assert!(public_method.is_some());
        assert!(public_method.unwrap().is_exported);

        let private_method = analyzer.find_binding("_private_method");
        assert!(private_method.is_some());
        assert!(!private_method.unwrap().is_exported);
    }
}
