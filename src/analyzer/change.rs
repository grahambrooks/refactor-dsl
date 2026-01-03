//! API change types and classifications.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use super::signature::{Parameter, TypeInfo};

/// Represents a single API change detected between versions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiChange {
    /// The kind of change.
    pub kind: ChangeKind,
    /// File path where the change occurred.
    pub file_path: PathBuf,
    /// Original identifier/signature (if applicable).
    pub original: Option<String>,
    /// New identifier/signature (if applicable).
    pub replacement: Option<String>,
    /// Confidence score (0.0 - 1.0) for fuzzy-matched changes.
    pub confidence: f64,
    /// Additional metadata about the change.
    pub metadata: ChangeMetadata,
}

impl ApiChange {
    /// Create a new API change with full confidence.
    pub fn new(kind: ChangeKind, file_path: PathBuf) -> Self {
        Self {
            kind,
            file_path,
            original: None,
            replacement: None,
            confidence: 1.0,
            metadata: ChangeMetadata::default(),
        }
    }

    /// Set the original value.
    pub fn with_original(mut self, original: impl Into<String>) -> Self {
        self.original = Some(original.into());
        self
    }

    /// Set the replacement value.
    pub fn with_replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = Some(replacement.into());
        self
    }

    /// Set the confidence score.
    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence;
        self
    }

    /// Set metadata.
    pub fn with_metadata(mut self, metadata: ChangeMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if this change requires manual review.
    pub fn requires_manual_review(&self) -> bool {
        matches!(
            self.kind,
            ChangeKind::SignatureChanged { .. }
                | ChangeKind::ParameterReordered { .. }
                | ChangeKind::ApiRemoved { .. }
        ) || self.confidence < 0.9
    }

    /// Check if this is a breaking change.
    pub fn is_breaking(&self) -> bool {
        self.metadata.severity == Severity::Breaking
    }
}

/// Classification of API changes.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
pub enum ChangeKind {
    /// Function/method was renamed.
    #[serde(rename = "function_renamed")]
    FunctionRenamed {
        old_name: String,
        new_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        module_path: Option<String>,
    },

    /// Import/module path changed.
    #[serde(rename = "import_renamed")]
    ImportRenamed { old_path: String, new_path: String },

    /// Function signature changed (parameters or return type).
    #[serde(rename = "signature_changed")]
    SignatureChanged {
        name: String,
        old_params: Vec<Parameter>,
        new_params: Vec<Parameter>,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_return: Option<TypeInfo>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_return: Option<TypeInfo>,
    },

    /// Parameter added to function.
    #[serde(rename = "parameter_added")]
    ParameterAdded {
        function_name: String,
        param_name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        param_type: Option<TypeInfo>,
        position: usize,
        has_default: bool,
    },

    /// Parameter removed from function.
    #[serde(rename = "parameter_removed")]
    ParameterRemoved {
        function_name: String,
        param_name: String,
        position: usize,
    },

    /// Parameters were reordered.
    #[serde(rename = "parameter_reordered")]
    ParameterReordered {
        function_name: String,
        old_order: Vec<String>,
        new_order: Vec<String>,
    },

    /// API was completely removed.
    #[serde(rename = "api_removed")]
    ApiRemoved { name: String, api_type: ApiType },

    /// Type/class/struct was renamed.
    #[serde(rename = "type_renamed")]
    TypeRenamed { old_name: String, new_name: String },

    /// Type definition changed.
    #[serde(rename = "type_changed")]
    TypeChanged { name: String, description: String },

    /// Method moved to different module/class.
    #[serde(rename = "method_moved")]
    MethodMoved {
        method_name: String,
        old_location: String,
        new_location: String,
    },

    /// Constant value changed.
    #[serde(rename = "constant_changed")]
    ConstantChanged {
        name: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        old_value: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_value: Option<String>,
    },
}

impl ChangeKind {
    /// Get a human-readable name for the change kind.
    pub fn name(&self) -> &'static str {
        match self {
            ChangeKind::FunctionRenamed { .. } => "Function Renamed",
            ChangeKind::ImportRenamed { .. } => "Import Path Changed",
            ChangeKind::SignatureChanged { .. } => "Signature Changed",
            ChangeKind::ParameterAdded { .. } => "Parameter Added",
            ChangeKind::ParameterRemoved { .. } => "Parameter Removed",
            ChangeKind::ParameterReordered { .. } => "Parameters Reordered",
            ChangeKind::ApiRemoved { .. } => "API Removed",
            ChangeKind::TypeRenamed { .. } => "Type Renamed",
            ChangeKind::TypeChanged { .. } => "Type Definition Changed",
            ChangeKind::MethodMoved { .. } => "Method Moved",
            ChangeKind::ConstantChanged { .. } => "Constant Changed",
        }
    }

    /// Check if this change kind can be automatically transformed.
    pub fn is_auto_transformable(&self) -> bool {
        matches!(
            self,
            ChangeKind::FunctionRenamed { .. }
                | ChangeKind::ImportRenamed { .. }
                | ChangeKind::TypeRenamed { .. }
                | ChangeKind::MethodMoved { .. }
        )
    }
}

/// Additional metadata about an API change.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ChangeMetadata {
    /// Line number in old version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_line: Option<usize>,
    /// Line number in new version.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_line: Option<usize>,
    /// Breaking change severity.
    #[serde(default)]
    pub severity: Severity,
    /// Suggested migration steps.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub migration_notes: Option<String>,
}

impl ChangeMetadata {
    /// Create metadata for a breaking change.
    pub fn breaking(notes: impl Into<String>) -> Self {
        Self {
            severity: Severity::Breaking,
            migration_notes: Some(notes.into()),
            ..Default::default()
        }
    }

    /// Create metadata for a warning-level change.
    pub fn warning(notes: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            migration_notes: Some(notes.into()),
            ..Default::default()
        }
    }

    /// Create metadata for an informational change.
    pub fn info(notes: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            migration_notes: Some(notes.into()),
            ..Default::default()
        }
    }
}

/// Severity level of an API change.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// Will cause compile/runtime errors - must be fixed.
    #[default]
    Breaking,
    /// May cause issues in some cases.
    Warning,
    /// Non-breaking but notable change.
    Info,
}

/// Type of API element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ApiType {
    Function,
    Method,
    Class,
    Struct,
    Enum,
    Constant,
    TypeAlias,
    Interface,
    Module,
    Trait,
}

impl ApiType {
    /// Get a human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            ApiType::Function => "function",
            ApiType::Method => "method",
            ApiType::Class => "class",
            ApiType::Struct => "struct",
            ApiType::Enum => "enum",
            ApiType::Constant => "constant",
            ApiType::TypeAlias => "type alias",
            ApiType::Interface => "interface",
            ApiType::Module => "module",
            ApiType::Trait => "trait",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_change_builder() {
        let change = ApiChange::new(
            ChangeKind::FunctionRenamed {
                old_name: "foo".into(),
                new_name: "bar".into(),
                module_path: None,
            },
            PathBuf::from("src/lib.rs"),
        )
        .with_original("foo")
        .with_replacement("bar")
        .with_confidence(0.95);

        assert_eq!(change.original, Some("foo".into()));
        assert_eq!(change.replacement, Some("bar".into()));
        assert!((change.confidence - 0.95).abs() < f64::EPSILON);
    }

    #[test]
    fn test_change_kind_names() {
        assert_eq!(
            ChangeKind::FunctionRenamed {
                old_name: "a".into(),
                new_name: "b".into(),
                module_path: None
            }
            .name(),
            "Function Renamed"
        );
        assert_eq!(
            ChangeKind::ApiRemoved {
                name: "x".into(),
                api_type: ApiType::Function
            }
            .name(),
            "API Removed"
        );
    }

    #[test]
    fn test_auto_transformable() {
        assert!(
            ChangeKind::FunctionRenamed {
                old_name: "a".into(),
                new_name: "b".into(),
                module_path: None
            }
            .is_auto_transformable()
        );

        assert!(
            !ChangeKind::SignatureChanged {
                name: "f".into(),
                old_params: vec![],
                new_params: vec![],
                old_return: None,
                new_return: None
            }
            .is_auto_transformable()
        );
    }

    #[test]
    fn test_requires_manual_review() {
        let auto_change = ApiChange::new(
            ChangeKind::FunctionRenamed {
                old_name: "a".into(),
                new_name: "b".into(),
                module_path: None,
            },
            PathBuf::from("test.rs"),
        );
        assert!(!auto_change.requires_manual_review());

        let manual_change = ApiChange::new(
            ChangeKind::ApiRemoved {
                name: "x".into(),
                api_type: ApiType::Function,
            },
            PathBuf::from("test.rs"),
        );
        assert!(manual_change.requires_manual_review());

        let low_confidence = ApiChange::new(
            ChangeKind::FunctionRenamed {
                old_name: "a".into(),
                new_name: "b".into(),
                module_path: None,
            },
            PathBuf::from("test.rs"),
        )
        .with_confidence(0.7);
        assert!(low_confidence.requires_manual_review());
    }
}
