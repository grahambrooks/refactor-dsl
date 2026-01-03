//! Binding tracking for variables, functions, and types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::lsp::Range;

/// The kind of binding (variable, function, type, etc.).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BindingKind {
    /// Local variable.
    Variable,
    /// Function parameter.
    Parameter,
    /// Function or method.
    Function,
    /// Method (member function).
    Method,
    /// Class definition.
    Class,
    /// Struct definition.
    Struct,
    /// Interface or trait definition.
    Interface,
    /// Enum definition.
    Enum,
    /// Module or namespace.
    Module,
    /// Constant value.
    Constant,
    /// Type alias.
    TypeAlias,
    /// Import/use statement.
    Import,
    /// Field in a struct/class.
    Field,
}

impl BindingKind {
    /// Returns a human-readable name for this binding kind.
    pub fn name(&self) -> &'static str {
        match self {
            BindingKind::Variable => "variable",
            BindingKind::Parameter => "parameter",
            BindingKind::Function => "function",
            BindingKind::Method => "method",
            BindingKind::Class => "class",
            BindingKind::Struct => "struct",
            BindingKind::Interface => "interface",
            BindingKind::Enum => "enum",
            BindingKind::Module => "module",
            BindingKind::Constant => "constant",
            BindingKind::TypeAlias => "type alias",
            BindingKind::Import => "import",
            BindingKind::Field => "field",
        }
    }

    /// Returns true if this binding is a type definition.
    pub fn is_type(&self) -> bool {
        matches!(
            self,
            BindingKind::Class
                | BindingKind::Struct
                | BindingKind::Interface
                | BindingKind::Enum
                | BindingKind::TypeAlias
        )
    }

    /// Returns true if this binding is callable.
    pub fn is_callable(&self) -> bool {
        matches!(self, BindingKind::Function | BindingKind::Method)
    }
}

/// A binding represents a named entity in the code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Binding {
    /// The name of the binding.
    pub name: String,
    /// The kind of binding.
    pub kind: BindingKind,
    /// The file where this binding is defined.
    pub file: PathBuf,
    /// The range in the source where this binding is defined.
    pub range: Range,
    /// The scope ID where this binding is visible.
    pub scope_id: ScopeId,
    /// Whether this binding is exported/public.
    pub is_exported: bool,
    /// Type annotation if available.
    pub type_annotation: Option<String>,
    /// Documentation comment if available.
    pub documentation: Option<String>,
}

impl Binding {
    /// Create a new binding.
    pub fn new(name: impl Into<String>, kind: BindingKind, file: PathBuf, range: Range) -> Self {
        Self {
            name: name.into(),
            kind,
            file,
            range,
            scope_id: ScopeId::root(),
            is_exported: false,
            type_annotation: None,
            documentation: None,
        }
    }

    /// Set the scope ID.
    pub fn with_scope(mut self, scope_id: ScopeId) -> Self {
        self.scope_id = scope_id;
        self
    }

    /// Mark as exported.
    pub fn exported(mut self) -> Self {
        self.is_exported = true;
        self
    }

    /// Set type annotation.
    pub fn with_type(mut self, type_annotation: impl Into<String>) -> Self {
        self.type_annotation = Some(type_annotation.into());
        self
    }

    /// Set documentation.
    pub fn with_docs(mut self, documentation: impl Into<String>) -> Self {
        self.documentation = Some(documentation.into());
        self
    }

    /// Get a unique identifier for this binding.
    pub fn unique_id(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.file.display(),
            self.range.start.line,
            self.range.start.character,
            self.name
        )
    }
}

/// A unique identifier for a scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ScopeId(pub u32);

impl ScopeId {
    /// The root/global scope.
    pub fn root() -> Self {
        Self(0)
    }

    /// Create a new scope ID.
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

/// The kind of scope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScopeKind {
    /// Global/module scope.
    Global,
    /// Function body.
    Function,
    /// Block scope (if, loop, etc.).
    Block,
    /// Class body.
    Class,
    /// Module/namespace scope.
    Module,
}

/// A scope represents a region where bindings are visible.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scope {
    /// Unique identifier for this scope.
    pub id: ScopeId,
    /// The kind of scope.
    pub kind: ScopeKind,
    /// Parent scope, if any.
    pub parent: Option<ScopeId>,
    /// The range this scope covers.
    pub range: Range,
    /// Bindings defined directly in this scope.
    binding_ids: Vec<String>,
}

impl Scope {
    /// Create a new scope.
    pub fn new(id: ScopeId, kind: ScopeKind, range: Range) -> Self {
        Self {
            id,
            kind,
            parent: None,
            range,
            binding_ids: Vec::new(),
        }
    }

    /// Set the parent scope.
    pub fn with_parent(mut self, parent: ScopeId) -> Self {
        self.parent = Some(parent);
        self
    }

    /// Add a binding to this scope.
    pub fn add_binding(&mut self, binding_id: String) {
        self.binding_ids.push(binding_id);
    }

    /// Get binding IDs in this scope.
    pub fn binding_ids(&self) -> &[String] {
        &self.binding_ids
    }
}

/// Tracks bindings and scopes for a file.
#[derive(Debug, Default)]
pub struct BindingTracker {
    /// All bindings indexed by unique ID.
    bindings: HashMap<String, Binding>,
    /// All scopes indexed by ID.
    scopes: HashMap<ScopeId, Scope>,
    /// Next scope ID to assign.
    next_scope_id: u32,
    /// Bindings by name for quick lookup.
    by_name: HashMap<String, Vec<String>>,
}

impl BindingTracker {
    /// Create a new binding tracker.
    pub fn new() -> Self {
        let mut tracker = Self::default();
        // Create the root scope
        tracker.scopes.insert(
            ScopeId::root(),
            Scope::new(
                ScopeId::root(),
                ScopeKind::Global,
                Range {
                    start: crate::lsp::Position {
                        line: 0,
                        character: 0,
                    },
                    end: crate::lsp::Position {
                        line: u32::MAX,
                        character: u32::MAX,
                    },
                },
            ),
        );
        tracker.next_scope_id = 1;
        tracker
    }

    /// Add a binding.
    pub fn add_binding(&mut self, binding: Binding) {
        let id = binding.unique_id();
        let name = binding.name.clone();
        let scope_id = binding.scope_id;

        self.bindings.insert(id.clone(), binding);
        self.by_name.entry(name).or_default().push(id.clone());

        if let Some(scope) = self.scopes.get_mut(&scope_id) {
            scope.add_binding(id);
        }
    }

    /// Create a new scope.
    pub fn create_scope(
        &mut self,
        kind: ScopeKind,
        range: Range,
        parent: Option<ScopeId>,
    ) -> ScopeId {
        let id = ScopeId::new(self.next_scope_id);
        self.next_scope_id += 1;

        let mut scope = Scope::new(id, kind, range);
        if let Some(parent_id) = parent {
            scope = scope.with_parent(parent_id);
        }

        self.scopes.insert(id, scope);
        id
    }

    /// Find bindings by name.
    pub fn find_by_name(&self, name: &str) -> Vec<&Binding> {
        self.by_name
            .get(name)
            .map(|ids| ids.iter().filter_map(|id| self.bindings.get(id)).collect())
            .unwrap_or_default()
    }

    /// Find a binding visible at a specific position within a scope.
    pub fn find_visible(&self, name: &str, at_scope: ScopeId) -> Option<&Binding> {
        let bindings = self.find_by_name(name);

        // Walk up the scope chain
        let mut current_scope = Some(at_scope);
        while let Some(scope_id) = current_scope {
            for binding in &bindings {
                if binding.scope_id == scope_id {
                    return Some(binding);
                }
            }

            current_scope = self.scopes.get(&scope_id).and_then(|s| s.parent);
        }

        None
    }

    /// Get all bindings.
    pub fn all_bindings(&self) -> impl Iterator<Item = &Binding> {
        self.bindings.values()
    }

    /// Get all exported bindings.
    pub fn exported_bindings(&self) -> impl Iterator<Item = &Binding> {
        self.bindings.values().filter(|b| b.is_exported)
    }

    /// Get a scope by ID.
    pub fn get_scope(&self, id: ScopeId) -> Option<&Scope> {
        self.scopes.get(&id)
    }

    /// Find the scope containing a position.
    pub fn scope_at(&self, line: u32, character: u32) -> Option<ScopeId> {
        // Find the most specific (innermost) scope containing this position
        let mut best_scope: Option<(ScopeId, u32)> = None; // (scope_id, depth)

        for (id, scope) in &self.scopes {
            if scope.range.start.line <= line
                && scope.range.end.line >= line
                && (scope.range.start.line < line || scope.range.start.character <= character)
                && (scope.range.end.line > line || scope.range.end.character >= character)
            {
                // Calculate depth
                let depth = self.scope_depth(*id);
                if best_scope.is_none_or(|(_, d)| depth > d) {
                    best_scope = Some((*id, depth));
                }
            }
        }

        best_scope.map(|(id, _)| id)
    }

    /// Calculate the depth of a scope (distance from root).
    fn scope_depth(&self, scope_id: ScopeId) -> u32 {
        let mut depth = 0;
        let mut current = Some(scope_id);

        while let Some(id) = current {
            if id == ScopeId::root() {
                break;
            }
            depth += 1;
            current = self.scopes.get(&id).and_then(|s| s.parent);
        }

        depth
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::Position;

    fn make_range(start_line: u32, end_line: u32) -> Range {
        Range {
            start: Position {
                line: start_line,
                character: 0,
            },
            end: Position {
                line: end_line,
                character: 100,
            },
        }
    }

    #[test]
    fn test_binding_creation() {
        let binding = Binding::new(
            "foo",
            BindingKind::Variable,
            PathBuf::from("test.rs"),
            make_range(1, 1),
        )
        .exported()
        .with_type("i32");

        assert_eq!(binding.name, "foo");
        assert_eq!(binding.kind, BindingKind::Variable);
        assert!(binding.is_exported);
        assert_eq!(binding.type_annotation, Some("i32".to_string()));
    }

    #[test]
    fn test_binding_tracker() {
        let mut tracker = BindingTracker::new();

        let binding = Binding::new(
            "my_var",
            BindingKind::Variable,
            PathBuf::from("test.rs"),
            make_range(5, 5),
        )
        .with_scope(ScopeId::root());

        tracker.add_binding(binding);

        let found = tracker.find_by_name("my_var");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "my_var");
    }

    #[test]
    fn test_scope_visibility() {
        let mut tracker = BindingTracker::new();

        // Create a function scope
        let fn_scope = tracker.create_scope(
            ScopeKind::Function,
            make_range(1, 10),
            Some(ScopeId::root()),
        );

        // Add a global binding
        let global_binding = Binding::new(
            "global_var",
            BindingKind::Variable,
            PathBuf::from("test.rs"),
            make_range(0, 0),
        )
        .with_scope(ScopeId::root());
        tracker.add_binding(global_binding);

        // Add a local binding
        let local_binding = Binding::new(
            "local_var",
            BindingKind::Variable,
            PathBuf::from("test.rs"),
            make_range(2, 2),
        )
        .with_scope(fn_scope);
        tracker.add_binding(local_binding);

        // Global should be visible from function scope
        assert!(tracker.find_visible("global_var", fn_scope).is_some());

        // Local should be visible from function scope
        assert!(tracker.find_visible("local_var", fn_scope).is_some());

        // Local should not be visible from global scope
        assert!(tracker.find_visible("local_var", ScopeId::root()).is_none());
    }

    #[test]
    fn test_binding_kind() {
        assert!(BindingKind::Class.is_type());
        assert!(BindingKind::Struct.is_type());
        assert!(!BindingKind::Function.is_type());

        assert!(BindingKind::Function.is_callable());
        assert!(BindingKind::Method.is_callable());
        assert!(!BindingKind::Variable.is_callable());
    }
}
