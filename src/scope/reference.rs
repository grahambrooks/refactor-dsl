//! Cross-file reference resolution.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::lsp::Range;

use super::binding::{Binding, BindingKind};

/// A reference to a binding from another location in the code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reference {
    /// The file containing this reference.
    pub file: PathBuf,
    /// The range of this reference in the source.
    pub range: Range,
    /// The name being referenced.
    pub name: String,
    /// The kind of reference (read, write, call, etc.).
    pub kind: ReferenceKind,
    /// Whether this is a definition (the binding itself).
    pub is_definition: bool,
}

impl Reference {
    /// Create a new reference.
    pub fn new(file: PathBuf, range: Range, name: impl Into<String>) -> Self {
        Self {
            file,
            range,
            name: name.into(),
            kind: ReferenceKind::Read,
            is_definition: false,
        }
    }

    /// Set the reference kind.
    pub fn with_kind(mut self, kind: ReferenceKind) -> Self {
        self.kind = kind;
        self
    }

    /// Mark as definition.
    pub fn as_definition(mut self) -> Self {
        self.is_definition = true;
        self
    }

    /// Get a unique identifier for this reference.
    pub fn unique_id(&self) -> String {
        format!(
            "{}:{}:{}",
            self.file.display(),
            self.range.start.line,
            self.range.start.character
        )
    }
}

/// The kind of reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReferenceKind {
    /// Reading a value.
    Read,
    /// Writing/assigning a value.
    Write,
    /// Calling a function/method.
    Call,
    /// Type reference (in type annotations).
    Type,
    /// Import/use reference.
    Import,
    /// Inheritance (extends, implements).
    Inheritance,
    /// Documentation reference.
    Documentation,
}

impl ReferenceKind {
    /// Returns true if this is a mutating reference.
    pub fn is_mutating(&self) -> bool {
        matches!(self, ReferenceKind::Write)
    }
}

/// Result of resolving a reference.
#[derive(Debug, Clone)]
pub struct ResolvedReference {
    /// The reference itself.
    pub reference: Reference,
    /// The binding it resolves to, if found.
    pub binding: Option<Binding>,
    /// Confidence level of the resolution.
    pub confidence: ResolutionConfidence,
}

/// Confidence level for reference resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ResolutionConfidence {
    /// No resolution found.
    None,
    /// Low confidence (name match only).
    Low,
    /// Medium confidence (name + scope).
    Medium,
    /// High confidence (name + scope + type).
    High,
    /// Certain (from LSP or exact match).
    Certain,
}

/// Index for cross-file reference resolution.
#[derive(Debug, Default)]
pub struct ReferenceIndex {
    /// All bindings indexed by name.
    bindings_by_name: HashMap<String, Vec<Binding>>,
    /// All bindings indexed by file.
    bindings_by_file: HashMap<PathBuf, Vec<Binding>>,
    /// All references indexed by file.
    references_by_file: HashMap<PathBuf, Vec<Reference>>,
    /// Cached resolutions.
    resolutions: HashMap<String, ResolvedReference>,
}

impl ReferenceIndex {
    /// Create a new reference index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a binding to the index.
    pub fn add_binding(&mut self, binding: Binding) {
        let name = binding.name.clone();
        let file = binding.file.clone();

        self.bindings_by_name
            .entry(name)
            .or_default()
            .push(binding.clone());
        self.bindings_by_file
            .entry(file)
            .or_default()
            .push(binding);
    }

    /// Add multiple bindings.
    pub fn add_bindings(&mut self, bindings: impl IntoIterator<Item = Binding>) {
        for binding in bindings {
            self.add_binding(binding);
        }
    }

    /// Add a reference to the index.
    pub fn add_reference(&mut self, reference: Reference) {
        self.references_by_file
            .entry(reference.file.clone())
            .or_default()
            .push(reference);
    }

    /// Add multiple references.
    pub fn add_references(&mut self, references: impl IntoIterator<Item = Reference>) {
        for reference in references {
            self.add_reference(reference);
        }
    }

    /// Find bindings by name.
    pub fn find_bindings(&self, name: &str) -> &[Binding] {
        self.bindings_by_name
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Find bindings in a file.
    pub fn bindings_in_file(&self, file: &Path) -> &[Binding] {
        self.bindings_by_file
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Find references in a file.
    pub fn references_in_file(&self, file: &Path) -> &[Reference] {
        self.references_by_file
            .get(file)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Resolve a reference to its binding.
    pub fn resolve(&mut self, reference: &Reference) -> ResolvedReference {
        let ref_id = reference.unique_id();

        // Check cache
        if let Some(cached) = self.resolutions.get(&ref_id) {
            return cached.clone();
        }

        let result = self.resolve_uncached(reference);

        // Cache the result
        self.resolutions.insert(ref_id, result.clone());

        result
    }

    /// Resolve without caching.
    fn resolve_uncached(&self, reference: &Reference) -> ResolvedReference {
        let candidates = self.find_bindings(&reference.name);

        if candidates.is_empty() {
            return ResolvedReference {
                reference: reference.clone(),
                binding: None,
                confidence: ResolutionConfidence::None,
            };
        }

        // Score candidates
        let mut best: Option<(Binding, ResolutionConfidence)> = None;

        for binding in candidates {
            let score = self.score_candidate(reference, binding);
            if best.as_ref().is_none_or(|(_, s)| score > *s) {
                best = Some((binding.clone(), score));
            }
        }

        match best {
            Some((binding, confidence)) => ResolvedReference {
                reference: reference.clone(),
                binding: Some(binding),
                confidence,
            },
            None => ResolvedReference {
                reference: reference.clone(),
                binding: None,
                confidence: ResolutionConfidence::None,
            },
        }
    }

    /// Score a candidate binding for a reference.
    fn score_candidate(&self, reference: &Reference, binding: &Binding) -> ResolutionConfidence {
        // Same file is a strong signal
        let same_file = reference.file == binding.file;

        // Check if reference kind matches binding kind
        let kind_match = match (&reference.kind, &binding.kind) {
            (ReferenceKind::Call, BindingKind::Function | BindingKind::Method) => true,
            (ReferenceKind::Type, k) if k.is_type() => true,
            (ReferenceKind::Read | ReferenceKind::Write, BindingKind::Variable | BindingKind::Parameter | BindingKind::Field) => true,
            (ReferenceKind::Import, BindingKind::Import) => true,
            (ReferenceKind::Inheritance, BindingKind::Class | BindingKind::Interface) => true,
            _ => false,
        };

        // Exported bindings are more likely targets for cross-file refs
        let export_match = !same_file && binding.is_exported;

        match (same_file, kind_match, export_match) {
            (true, true, _) => ResolutionConfidence::High,
            (true, false, _) => ResolutionConfidence::Medium,
            (false, true, true) => ResolutionConfidence::Medium,
            (false, true, false) => ResolutionConfidence::Low,
            (false, false, _) => ResolutionConfidence::Low,
        }
    }

    /// Find all references to a binding.
    pub fn find_references_to(&self, binding: &Binding) -> Vec<&Reference> {
        let mut refs = Vec::new();

        for references in self.references_by_file.values() {
            for reference in references {
                if reference.name == binding.name {
                    refs.push(reference);
                }
            }
        }

        refs
    }

    /// Get all files in the index.
    pub fn files(&self) -> impl Iterator<Item = &PathBuf> {
        self.bindings_by_file
            .keys()
            .chain(self.references_by_file.keys())
    }

    /// Clear the resolution cache.
    pub fn clear_cache(&mut self) {
        self.resolutions.clear();
    }

    /// Clear all data.
    pub fn clear(&mut self) {
        self.bindings_by_name.clear();
        self.bindings_by_file.clear();
        self.references_by_file.clear();
        self.resolutions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::Position;

    fn make_range(line: u32) -> Range {
        Range {
            start: Position { line, character: 0 },
            end: Position {
                line,
                character: 10,
            },
        }
    }

    #[test]
    fn test_reference_creation() {
        let reference = Reference::new(PathBuf::from("test.rs"), make_range(5), "foo")
            .with_kind(ReferenceKind::Call);

        assert_eq!(reference.name, "foo");
        assert_eq!(reference.kind, ReferenceKind::Call);
        assert!(!reference.is_definition);
    }

    #[test]
    fn test_reference_index_add_binding() {
        let mut index = ReferenceIndex::new();

        let binding = Binding::new(
            "my_func",
            BindingKind::Function,
            PathBuf::from("lib.rs"),
            make_range(10),
        )
        .exported();

        index.add_binding(binding);

        let found = index.find_bindings("my_func");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].name, "my_func");
    }

    #[test]
    fn test_reference_resolution() {
        let mut index = ReferenceIndex::new();

        // Add a function binding
        let binding = Binding::new(
            "helper",
            BindingKind::Function,
            PathBuf::from("utils.rs"),
            make_range(5),
        )
        .exported();
        index.add_binding(binding);

        // Add a call reference
        let reference = Reference::new(PathBuf::from("main.rs"), make_range(20), "helper")
            .with_kind(ReferenceKind::Call);

        let resolved = index.resolve(&reference);
        assert!(resolved.binding.is_some());
        assert_eq!(resolved.binding.unwrap().name, "helper");
        assert!(resolved.confidence >= ResolutionConfidence::Low);
    }

    #[test]
    fn test_same_file_resolution() {
        let mut index = ReferenceIndex::new();

        // Add a variable binding
        let binding = Binding::new(
            "counter",
            BindingKind::Variable,
            PathBuf::from("main.rs"),
            make_range(5),
        );
        index.add_binding(binding);

        // Add a read reference in the same file
        let reference = Reference::new(PathBuf::from("main.rs"), make_range(10), "counter")
            .with_kind(ReferenceKind::Read);

        let resolved = index.resolve(&reference);
        assert!(resolved.binding.is_some());
        assert!(resolved.confidence >= ResolutionConfidence::Medium);
    }

    #[test]
    fn test_find_references_to() {
        let mut index = ReferenceIndex::new();

        let binding = Binding::new(
            "shared_func",
            BindingKind::Function,
            PathBuf::from("lib.rs"),
            make_range(1),
        );
        index.add_binding(binding.clone());

        // Add references from multiple files
        index.add_reference(Reference::new(
            PathBuf::from("main.rs"),
            make_range(10),
            "shared_func",
        ));
        index.add_reference(Reference::new(
            PathBuf::from("other.rs"),
            make_range(20),
            "shared_func",
        ));
        index.add_reference(Reference::new(
            PathBuf::from("main.rs"),
            make_range(30),
            "other_func",
        ));

        let refs = index.find_references_to(&binding);
        assert_eq!(refs.len(), 2);
    }
}
