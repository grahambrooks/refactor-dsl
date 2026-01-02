//! Usage analysis for safe refactoring operations.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use super::binding::{Binding, BindingKind};
use super::reference::{Reference, ReferenceIndex, ResolutionConfidence};

/// Result of usage analysis for a binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageInfo {
    /// The binding being analyzed.
    pub binding_name: String,
    /// The file where the binding is defined.
    pub definition_file: PathBuf,
    /// Total number of usages found.
    pub usage_count: usize,
    /// Number of files using this binding.
    pub file_count: usize,
    /// Breakdown by reference kind.
    pub by_kind: HashMap<String, usize>,
    /// Files that use this binding.
    pub used_in_files: Vec<PathBuf>,
    /// Whether this binding appears unused.
    pub is_unused: bool,
    /// Whether this binding is only used internally (same file).
    pub is_internal_only: bool,
}

impl UsageInfo {
    /// Create new usage info for a binding.
    pub fn new(binding: &Binding) -> Self {
        Self {
            binding_name: binding.name.clone(),
            definition_file: binding.file.clone(),
            usage_count: 0,
            file_count: 0,
            by_kind: HashMap::new(),
            used_in_files: Vec::new(),
            is_unused: true,
            is_internal_only: true,
        }
    }

    /// Add a usage.
    pub fn add_usage(&mut self, reference: &Reference) {
        self.usage_count += 1;
        self.is_unused = false;

        let kind_name = format!("{:?}", reference.kind);
        *self.by_kind.entry(kind_name).or_insert(0) += 1;

        if !self.used_in_files.contains(&reference.file) {
            self.used_in_files.push(reference.file.clone());
            self.file_count += 1;

            if reference.file != self.definition_file {
                self.is_internal_only = false;
            }
        }
    }
}

/// Analysis of potentially dead (unused) code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeadCodeInfo {
    /// Unused bindings.
    pub unused_bindings: Vec<UnusedBinding>,
    /// Total count of unused items.
    pub total_unused: usize,
    /// Breakdown by kind.
    pub by_kind: HashMap<String, usize>,
}

/// An unused binding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnusedBinding {
    /// The binding name.
    pub name: String,
    /// The kind of binding.
    pub kind: String,
    /// The file where it's defined.
    pub file: PathBuf,
    /// Line number.
    pub line: u32,
    /// Whether it's exported (might be used externally).
    pub is_exported: bool,
    /// Confidence that it's truly unused.
    pub confidence: String,
}

/// Analyzes usage patterns in code.
#[derive(Debug, Default)]
pub struct UsageAnalyzer {
    /// The reference index.
    index: ReferenceIndex,
    /// Cached usage info.
    usage_cache: HashMap<String, UsageInfo>,
}

impl UsageAnalyzer {
    /// Create a new usage analyzer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from an existing reference index.
    pub fn with_index(index: ReferenceIndex) -> Self {
        Self {
            index,
            usage_cache: HashMap::new(),
        }
    }

    /// Get the reference index.
    pub fn index(&self) -> &ReferenceIndex {
        &self.index
    }

    /// Get mutable reference index.
    pub fn index_mut(&mut self) -> &mut ReferenceIndex {
        &mut self.index
    }

    /// Add a binding to analyze.
    pub fn add_binding(&mut self, binding: Binding) {
        self.index.add_binding(binding);
        self.usage_cache.clear();
    }

    /// Add a reference.
    pub fn add_reference(&mut self, reference: Reference) {
        self.index.add_reference(reference);
        self.usage_cache.clear();
    }

    /// Analyze usage for a specific binding.
    pub fn analyze_binding(&mut self, binding: &Binding) -> UsageInfo {
        let binding_id = binding.unique_id();

        if let Some(cached) = self.usage_cache.get(&binding_id) {
            return cached.clone();
        }

        let mut info = UsageInfo::new(binding);

        // Find all references to this binding
        let refs = self.index.find_references_to(binding);
        for reference in refs {
            if !reference.is_definition {
                info.add_usage(reference);
            }
        }

        self.usage_cache.insert(binding_id, info.clone());
        info
    }

    /// Analyze all bindings and find unused ones.
    pub fn find_dead_code(&mut self) -> DeadCodeInfo {
        let mut unused = Vec::new();
        let mut by_kind: HashMap<String, usize> = HashMap::new();

        // Collect all bindings first to avoid borrow issues
        let bindings: Vec<_> = self
            .index
            .files()
            .flat_map(|file| self.index.bindings_in_file(file).to_vec())
            .collect();

        for binding in bindings {
            let info = self.analyze_binding(&binding);

            if info.is_unused {
                let kind_name = format!("{:?}", binding.kind);
                *by_kind.entry(kind_name.clone()).or_insert(0) += 1;

                // Determine confidence
                let confidence = if binding.is_exported {
                    "low" // Exported items might be used externally
                } else if binding.kind == BindingKind::Parameter {
                    "medium" // Parameters might be required by interface
                } else {
                    "high"
                };

                unused.push(UnusedBinding {
                    name: binding.name.clone(),
                    kind: kind_name,
                    file: binding.file.clone(),
                    line: binding.range.start.line,
                    is_exported: binding.is_exported,
                    confidence: confidence.to_string(),
                });
            }
        }

        DeadCodeInfo {
            total_unused: unused.len(),
            unused_bindings: unused,
            by_kind,
        }
    }

    /// Check if a binding can be safely deleted.
    pub fn can_safely_delete(&mut self, binding: &Binding) -> SafeDeleteResult {
        let info = self.analyze_binding(binding);

        if info.is_unused {
            return SafeDeleteResult {
                can_delete: true,
                reason: None,
                blockers: Vec::new(),
            };
        }

        // Collect blocking references
        let blockers: Vec<_> = self
            .index
            .find_references_to(binding)
            .iter()
            .filter(|r| !r.is_definition)
            .map(|r| DeleteBlocker {
                file: r.file.clone(),
                line: r.range.start.line,
                kind: format!("{:?}", r.kind),
            })
            .collect();

        SafeDeleteResult {
            can_delete: false,
            reason: Some(format!(
                "Binding '{}' has {} usage(s) in {} file(s)",
                binding.name, info.usage_count, info.file_count
            )),
            blockers,
        }
    }

    /// Find all usages of a binding.
    pub fn find_all_usages(&self, binding: &Binding) -> Vec<&Reference> {
        self.index.find_references_to(binding)
    }

    /// Get files that depend on a specific file.
    pub fn files_depending_on(&self, file: &Path) -> HashSet<PathBuf> {
        let bindings_in_file = self.index.bindings_in_file(file);
        let mut dependents = HashSet::new();

        for binding in bindings_in_file {
            for reference in self.index.find_references_to(binding) {
                if reference.file != file {
                    dependents.insert(reference.file.clone());
                }
            }
        }

        dependents
    }

    /// Get files that a specific file depends on.
    pub fn file_dependencies(&mut self, file: &Path) -> HashSet<PathBuf> {
        // Collect references first to avoid borrow issues
        let references: Vec<_> = self.index.references_in_file(file).to_vec();
        let mut dependencies = HashSet::new();

        for reference in &references {
            let resolved = self.index.resolve(reference);
            if let Some(binding) = resolved.binding {
                if binding.file != file && resolved.confidence >= ResolutionConfidence::Medium {
                    dependencies.insert(binding.file.clone());
                }
            }
        }

        dependencies
    }

    /// Clear the usage cache.
    pub fn clear_cache(&mut self) {
        self.usage_cache.clear();
        self.index.clear_cache();
    }
}

/// Result of checking if a binding can be safely deleted.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafeDeleteResult {
    /// Whether the binding can be deleted.
    pub can_delete: bool,
    /// Reason if it cannot be deleted.
    pub reason: Option<String>,
    /// References blocking deletion.
    pub blockers: Vec<DeleteBlocker>,
}

/// A reference blocking deletion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteBlocker {
    /// File containing the blocking reference.
    pub file: PathBuf,
    /// Line number.
    pub line: u32,
    /// Kind of reference.
    pub kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lsp::{Position, Range};
    use crate::scope::reference::ReferenceKind;

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
    fn test_usage_info() {
        let binding = Binding::new(
            "test_func",
            BindingKind::Function,
            PathBuf::from("lib.rs"),
            make_range(5),
        );

        let mut info = UsageInfo::new(&binding);
        assert!(info.is_unused);
        assert!(info.is_internal_only);

        // Add a usage from the same file
        let ref1 = Reference::new(PathBuf::from("lib.rs"), make_range(10), "test_func");
        info.add_usage(&ref1);

        assert!(!info.is_unused);
        assert!(info.is_internal_only);
        assert_eq!(info.usage_count, 1);

        // Add a usage from another file
        let ref2 = Reference::new(PathBuf::from("main.rs"), make_range(20), "test_func");
        info.add_usage(&ref2);

        assert!(!info.is_internal_only);
        assert_eq!(info.usage_count, 2);
        assert_eq!(info.file_count, 2);
    }

    #[test]
    fn test_find_dead_code() {
        let mut analyzer = UsageAnalyzer::new();

        // Add an unused function
        let unused_func = Binding::new(
            "unused_helper",
            BindingKind::Function,
            PathBuf::from("utils.rs"),
            make_range(10),
        );
        analyzer.add_binding(unused_func);

        // Add a used function
        let used_func = Binding::new(
            "used_helper",
            BindingKind::Function,
            PathBuf::from("utils.rs"),
            make_range(20),
        );
        analyzer.add_binding(used_func);

        // Add a reference to the used function
        let reference = Reference::new(PathBuf::from("main.rs"), make_range(5), "used_helper")
            .with_kind(ReferenceKind::Call);
        analyzer.add_reference(reference);

        let dead_code = analyzer.find_dead_code();

        // Only the unused function should be reported
        assert_eq!(dead_code.total_unused, 1);
        assert_eq!(dead_code.unused_bindings[0].name, "unused_helper");
    }

    #[test]
    fn test_safe_delete() {
        let mut analyzer = UsageAnalyzer::new();

        // Add a function
        let func = Binding::new(
            "deletable",
            BindingKind::Function,
            PathBuf::from("lib.rs"),
            make_range(5),
        );
        analyzer.add_binding(func.clone());

        // Should be safe to delete (no usages)
        let result = analyzer.can_safely_delete(&func);
        assert!(result.can_delete);

        // Add a reference
        let reference = Reference::new(PathBuf::from("main.rs"), make_range(10), "deletable");
        analyzer.add_reference(reference);
        analyzer.clear_cache();

        // Now should not be safe to delete
        let result = analyzer.can_safely_delete(&func);
        assert!(!result.can_delete);
        assert_eq!(result.blockers.len(), 1);
    }

    #[test]
    fn test_file_dependencies() {
        let mut analyzer = UsageAnalyzer::new();

        // Add bindings in utils.rs
        let util_func = Binding::new(
            "helper",
            BindingKind::Function,
            PathBuf::from("utils.rs"),
            make_range(5),
        )
        .exported();
        analyzer.add_binding(util_func);

        // Add reference from main.rs to utils.rs
        let reference = Reference::new(PathBuf::from("main.rs"), make_range(10), "helper")
            .with_kind(ReferenceKind::Call);
        analyzer.add_reference(reference);

        // main.rs depends on utils.rs
        let deps = analyzer.file_dependencies(Path::new("main.rs"));
        assert!(deps.contains(&PathBuf::from("utils.rs")));

        // utils.rs has main.rs as dependent
        let dependents = analyzer.files_depending_on(Path::new("utils.rs"));
        assert!(dependents.contains(&PathBuf::from("main.rs")));
    }
}
