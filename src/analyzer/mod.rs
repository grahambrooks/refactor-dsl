//! Library change analyzer and codemod generator.
//!
//! This module provides tools for analyzing API changes between library versions
//! and automatically generating codemods to upgrade dependent projects.
//!
//! # Overview
//!
//! The analyzer works in three phases:
//!
//! 1. **Extraction**: Parse source files at two git refs and extract API signatures
//! 2. **Detection**: Compare signatures to detect renames, removals, signature changes
//! 3. **Generation**: Convert detected changes to transforms for dependent projects
//!
//! # Example
//!
//! ```rust,no_run
//! use refactor_dsl::analyzer::LibraryAnalyzer;
//! use refactor_dsl::codemod::Codemod;
//!
//! // Analyze library changes between versions
//! let analyzer = LibraryAnalyzer::new("./my-library")?;
//! let upgrade = analyzer.generate_upgrade("v1.0.0", "v2.0.0")?;
//!
//! // Print a report of detected changes
//! println!("{}", upgrade.report());
//!
//! // Apply to dependent projects
//! Codemod::from_local("./projects")
//!     .apply(upgrade)
//!     .execute()?;
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```
//!
//! # Configuration Files
//!
//! You can also save and load upgrade configurations:
//!
//! ```rust,no_run
//! use refactor_dsl::analyzer::{LibraryAnalyzer, UpgradeConfig};
//!
//! // Generate and save config
//! let analyzer = LibraryAnalyzer::new("./my-library")?;
//! let config = analyzer.analyze_to_config("v1.0.0", "v2.0.0")?;
//! config.to_yaml("upgrade.yaml")?;
//!
//! // Load and apply config later
//! let loaded = UpgradeConfig::from_yaml("upgrade.yaml")?;
//! let upgrade = loaded.to_upgrade();
//! # Ok::<(), refactor_dsl::error::RefactorError>(())
//! ```

mod change;
mod config;
mod detector;
mod extractor;
mod generator;
mod signature;

pub use change::{ApiChange, ApiType, ChangeKind, ChangeMetadata, Severity};
pub use config::{ConfigBasedUpgrade, TransformSpec, UpgradeConfig};
pub use detector::ChangeDetector;
pub use extractor::{ApiExtractor, FileChange, FileChangeType, FileContent, GitDiffReader};
pub use generator::{GeneratedUpgrade, Transform, UpgradeGenerator};
pub use signature::{ApiSignature, Parameter, SourceLocation, TypeInfo, Visibility};

use crate::error::{RefactorError, Result};
use crate::lang::LanguageRegistry;
use git2::Repository;
use std::path::{Path, PathBuf};

/// Result of analyzing a library between two versions.
#[derive(Debug)]
pub struct AnalysisResult {
    /// The detected API changes.
    pub changes: Vec<ApiChange>,
    /// Files that changed between versions.
    pub changed_files: Vec<FileChange>,
    /// From ref/version.
    pub from_ref: String,
    /// To ref/version.
    pub to_ref: String,
}

impl AnalysisResult {
    /// Get breaking changes.
    pub fn breaking_changes(&self) -> Vec<&ApiChange> {
        self.changes.iter().filter(|c| c.is_breaking()).collect()
    }

    /// Get changes that can be auto-transformed.
    pub fn auto_transformable(&self) -> Vec<&ApiChange> {
        self.changes
            .iter()
            .filter(|c| c.kind.is_auto_transformable())
            .collect()
    }

    /// Get changes that require manual review.
    pub fn manual_review(&self) -> Vec<&ApiChange> {
        self.changes
            .iter()
            .filter(|c| c.requires_manual_review())
            .collect()
    }
}

/// Analyzes API changes between library versions and generates upgrade codemods.
///
/// The `LibraryAnalyzer` opens a git repository and compares API signatures
/// between two refs (tags, branches, or commits) to detect changes.
///
/// # Example
///
/// ```rust,no_run
/// use refactor_dsl::analyzer::LibraryAnalyzer;
///
/// let analyzer = LibraryAnalyzer::new("./my-library")?
///     .for_extensions(vec!["ts", "tsx"])
///     .rename_threshold(0.8);
///
/// let result = analyzer.analyze("v1.0.0", "v2.0.0")?;
///
/// println!("Found {} API changes", result.changes.len());
/// println!("Breaking: {}", result.breaking_changes().len());
/// println!("Auto-fixable: {}", result.auto_transformable().len());
/// # Ok::<(), refactor_dsl::error::RefactorError>(())
/// ```
pub struct LibraryAnalyzer {
    repo_path: PathBuf,
    repo: Repository,
    extensions: Vec<String>,
    registry: LanguageRegistry,
    rename_threshold: f64,
    include_private: bool,
}

impl LibraryAnalyzer {
    /// Create a new analyzer for a git repository.
    pub fn new(repo_path: impl AsRef<Path>) -> Result<Self> {
        let repo_path = repo_path.as_ref().to_path_buf();
        let repo = Repository::open(&repo_path).map_err(|e| {
            RefactorError::InvalidConfig(format!(
                "Failed to open repository at {:?}: {}",
                repo_path, e
            ))
        })?;

        Ok(Self {
            repo_path,
            repo,
            extensions: vec![
                "rs".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "py".to_string(),
            ],
            registry: LanguageRegistry::new(),
            rename_threshold: 0.7,
            include_private: false,
        })
    }

    /// Filter analysis to specific file extensions.
    pub fn for_extensions(mut self, extensions: Vec<&str>) -> Self {
        self.extensions = extensions.into_iter().map(String::from).collect();
        self
    }

    /// Set the similarity threshold for rename detection (0.0 - 1.0).
    ///
    /// Higher values require more similarity to consider something a rename
    /// rather than a removal and addition.
    pub fn rename_threshold(mut self, threshold: f64) -> Self {
        self.rename_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Include private (non-exported) APIs in the analysis.
    pub fn include_private(mut self, include: bool) -> Self {
        self.include_private = include;
        self
    }

    /// Analyze API changes between two git refs.
    ///
    /// The refs can be tags (e.g., "v1.0.0"), branches, or commit hashes.
    pub fn analyze(&self, from_ref: &str, to_ref: &str) -> Result<AnalysisResult> {
        // Read files at both refs
        let diff_reader = GitDiffReader::new(&self.repo)
            .filter_extensions(self.extensions.clone());

        let changed_files = diff_reader.changed_files(from_ref, to_ref)?;

        // Get all files at each ref
        let old_files = diff_reader.files_at_ref(from_ref)?;
        let new_files = diff_reader.files_at_ref(to_ref)?;

        // Extract API signatures
        let extractor = ApiExtractor::with_registry(LanguageRegistry::new());
        let old_apis = extractor.extract_all(&old_files)?;
        let new_apis = extractor.extract_all(&new_files)?;

        // Detect changes
        let detector = ChangeDetector::new()
            .rename_threshold(self.rename_threshold)
            .include_private(self.include_private);

        let changes = detector.detect(&old_apis, &new_apis);

        Ok(AnalysisResult {
            changes,
            changed_files,
            from_ref: from_ref.to_string(),
            to_ref: to_ref.to_string(),
        })
    }

    /// Analyze and generate an upgrade that can be applied to dependent projects.
    pub fn generate_upgrade(&self, from_ref: &str, to_ref: &str) -> Result<GeneratedUpgrade> {
        let analysis = self.analyze(from_ref, to_ref)?;

        let name = format!(
            "{}-{}-to-{}",
            self.repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("library"),
            sanitize_version(from_ref),
            sanitize_version(to_ref)
        );

        let description = format!(
            "Upgrade {} from {} to {}",
            self.repo_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("library"),
            from_ref,
            to_ref
        );

        Ok(UpgradeGenerator::new(name, description)
            .with_changes(analysis.changes)
            .for_extensions(self.extensions.clone())
            .generate())
    }

    /// Analyze and generate a serializable upgrade configuration.
    pub fn analyze_to_config(&self, from_ref: &str, to_ref: &str) -> Result<UpgradeConfig> {
        let upgrade = self.generate_upgrade(from_ref, to_ref)?;

        let mut config = UpgradeConfig::new(&upgrade.name, &upgrade.description)
            .with_extensions(self.extensions.clone())
            .with_versions(from_ref, to_ref);

        // Convert transforms to specs
        for transform in &upgrade.transforms {
            let spec = match transform {
                Transform::FunctionRename { old_name, new_name } => {
                    TransformSpec::RenameFunction {
                        old_name: old_name.clone(),
                        new_name: new_name.clone(),
                    }
                }
                Transform::TypeRename { old_name, new_name } => TransformSpec::RenameType {
                    old_name: old_name.clone(),
                    new_name: new_name.clone(),
                },
                Transform::ImportRename { old_path, new_path } => TransformSpec::RenameImport {
                    old_path: old_path.clone(),
                    new_path: new_path.clone(),
                },
                Transform::MethodMove { .. } | Transform::ConstantUpdate { .. } => {
                    let (pattern, replacement) = transform.to_pattern_replacement();
                    TransformSpec::ReplacePattern { pattern, replacement }
                }
            };
            config.add_transform(spec);
        }

        // Include original changes for reference
        config.changes = upgrade.changes;

        Ok(config)
    }

    /// Get the repository path.
    pub fn repo_path(&self) -> &Path {
        &self.repo_path
    }
}

/// Sanitize a version string for use in names.
fn sanitize_version(version: &str) -> String {
    version
        .trim_start_matches('v')
        .replace('.', "-")
        .replace('/', "-")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_version() {
        assert_eq!(sanitize_version("v1.0.0"), "1-0-0");
        assert_eq!(sanitize_version("1.2.3"), "1-2-3");
        assert_eq!(sanitize_version("feature/test"), "feature-test");
    }

    #[test]
    fn test_analysis_result_filters() {
        let changes = vec![
            ApiChange::new(
                ChangeKind::FunctionRenamed {
                    old_name: "old".into(),
                    new_name: "new".into(),
                    module_path: None,
                },
                PathBuf::from("lib.rs"),
            ),
            ApiChange::new(
                ChangeKind::ApiRemoved {
                    name: "removed".into(),
                    api_type: ApiType::Function,
                },
                PathBuf::from("lib.rs"),
            ),
        ];

        let result = AnalysisResult {
            changes,
            changed_files: vec![],
            from_ref: "v1".into(),
            to_ref: "v2".into(),
        };

        assert_eq!(result.breaking_changes().len(), 2);
        assert_eq!(result.auto_transformable().len(), 1);
        assert_eq!(result.manual_review().len(), 1);
    }
}
