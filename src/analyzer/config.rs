//! Serializable configuration for upgrade definitions.

use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::codemod::Upgrade;
use crate::error::{RefactorError, Result};
use crate::matcher::Matcher;
use crate::transform::TransformBuilder;

use super::change::ApiChange;

/// A serializable specification for a transform.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TransformSpec {
    /// Replace a literal string with another.
    #[serde(rename = "replace_literal")]
    ReplaceLiteral { from: String, to: String },

    /// Replace using a regex pattern.
    #[serde(rename = "replace_pattern")]
    ReplacePattern { pattern: String, replacement: String },

    /// Rename a function (generates appropriate pattern).
    #[serde(rename = "rename_function")]
    RenameFunction { old_name: String, new_name: String },

    /// Rename a type (generates appropriate pattern).
    #[serde(rename = "rename_type")]
    RenameType { old_name: String, new_name: String },

    /// Update an import path.
    #[serde(rename = "rename_import")]
    RenameImport { old_path: String, new_path: String },
}

impl TransformSpec {
    /// Convert this spec to a pattern and replacement.
    pub fn to_pattern_replacement(&self) -> (String, String) {
        match self {
            TransformSpec::ReplaceLiteral { from, to } => (regex::escape(from), to.clone()),

            TransformSpec::ReplacePattern {
                pattern,
                replacement,
            } => (pattern.clone(), replacement.clone()),

            TransformSpec::RenameFunction { old_name, new_name } => {
                let pattern = format!(r"\b{}\s*(\(|::<)", regex::escape(old_name));
                let replacement = format!("{}$1", new_name);
                (pattern, replacement)
            }

            TransformSpec::RenameType { old_name, new_name } => {
                let pattern = format!(r"\b{}\b", regex::escape(old_name));
                (pattern, new_name.clone())
            }

            TransformSpec::RenameImport { old_path, new_path } => {
                let pattern = format!(r#"(['"]){}\1"#, regex::escape(old_path));
                let replacement = format!("$1{}$1", new_path);
                (pattern, replacement)
            }
        }
    }
}

/// A serializable upgrade configuration.
///
/// Can be saved to and loaded from YAML or JSON files.
///
/// # Example YAML
///
/// ```yaml
/// name: my-library-v2
/// description: Upgrade to my-library v2.0
/// extensions:
///   - ts
///   - tsx
/// transforms:
///   - type: rename_function
///     old_name: getData
///     new_name: fetchData
///   - type: rename_type
///     old_name: UserData
///     new_name: User
///   - type: replace_literal
///     from: "deprecated_api"
///     to: "new_api"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpgradeConfig {
    /// Unique name for this upgrade.
    pub name: String,

    /// Human-readable description.
    pub description: String,

    /// File extensions to target (e.g., ["ts", "rs"]).
    #[serde(default)]
    pub extensions: Vec<String>,

    /// Glob patterns to exclude.
    #[serde(default)]
    pub exclude_patterns: Vec<String>,

    /// The transforms to apply.
    pub transforms: Vec<TransformSpec>,

    /// Original detected changes (optional, for reference).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<ApiChange>,

    /// Library version this upgrade is from.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from_version: Option<String>,

    /// Library version this upgrade is to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to_version: Option<String>,
}

impl Default for UpgradeConfig {
    fn default() -> Self {
        Self {
            name: "unnamed-upgrade".to_string(),
            description: "No description".to_string(),
            extensions: vec!["ts".to_string(), "rs".to_string(), "py".to_string()],
            exclude_patterns: vec![
                "**/node_modules/**".to_string(),
                "**/target/**".to_string(),
                "**/.git/**".to_string(),
            ],
            transforms: Vec::new(),
            changes: Vec::new(),
            from_version: None,
            to_version: None,
        }
    }
}

impl UpgradeConfig {
    /// Create a new upgrade config.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            ..Default::default()
        }
    }

    /// Add a transform specification.
    pub fn add_transform(&mut self, transform: TransformSpec) {
        self.transforms.push(transform);
    }

    /// Set target extensions.
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Set exclude patterns.
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    /// Set version information.
    pub fn with_versions(
        mut self,
        from: impl Into<String>,
        to: impl Into<String>,
    ) -> Self {
        self.from_version = Some(from.into());
        self.to_version = Some(to.into());
        self
    }

    /// Load config from a YAML file.
    pub fn from_yaml(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            RefactorError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read config file: {}", e),
            ))
        })?;

        serde_yaml::from_str(&content).map_err(|e| {
            RefactorError::InvalidConfig(format!("Failed to parse YAML config: {}", e))
        })
    }

    /// Load config from a JSON file.
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            RefactorError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to read config file: {}", e),
            ))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            RefactorError::InvalidConfig(format!("Failed to parse JSON config: {}", e))
        })
    }

    /// Save config to a YAML file.
    pub fn to_yaml(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_yaml::to_string(self).map_err(|e| {
            RefactorError::InvalidConfig(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path.as_ref(), content).map_err(|e| {
            RefactorError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write config file: {}", e),
            ))
        })
    }

    /// Save config to a JSON file.
    pub fn to_json(&self, path: impl AsRef<Path>) -> Result<()> {
        let content = serde_json::to_string_pretty(self).map_err(|e| {
            RefactorError::InvalidConfig(format!("Failed to serialize config: {}", e))
        })?;

        std::fs::write(path.as_ref(), content).map_err(|e| {
            RefactorError::Io(std::io::Error::new(
                e.kind(),
                format!("Failed to write config file: {}", e),
            ))
        })
    }

    /// Convert to an Upgrade implementation.
    pub fn to_upgrade(&self) -> ConfigBasedUpgrade {
        ConfigBasedUpgrade {
            config: self.clone(),
        }
    }
}

/// An Upgrade implementation based on a config file.
#[derive(Debug, Clone)]
pub struct ConfigBasedUpgrade {
    config: UpgradeConfig,
}

impl ConfigBasedUpgrade {
    /// Create from a config.
    pub fn new(config: UpgradeConfig) -> Self {
        Self { config }
    }

    /// Load from a YAML file.
    pub fn from_yaml(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::new(UpgradeConfig::from_yaml(path)?))
    }

    /// Load from a JSON file.
    pub fn from_json(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self::new(UpgradeConfig::from_json(path)?))
    }

    /// Get the underlying config.
    pub fn config(&self) -> &UpgradeConfig {
        &self.config
    }
}

impl Upgrade for ConfigBasedUpgrade {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn description(&self) -> &str {
        &self.config.description
    }

    fn matcher(&self) -> Matcher {
        let extensions: Vec<&str> = self.config.extensions.iter().map(|s| s.as_str()).collect();
        let exclude_patterns = self.config.exclude_patterns.clone();

        if extensions.is_empty() {
            Matcher::new()
        } else {
            Matcher::new().files(move |mut f| {
                f = f.extensions(extensions.clone());
                for pattern in &exclude_patterns {
                    f = f.exclude(pattern.as_str());
                }
                f
            })
        }
    }

    fn transform(&self) -> TransformBuilder {
        let mut builder = TransformBuilder::new();

        for spec in &self.config.transforms {
            let (pattern, replacement) = spec.to_pattern_replacement();
            builder = builder.replace_pattern(&pattern, &replacement);
        }

        builder
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_spec_rename_function() {
        let spec = TransformSpec::RenameFunction {
            old_name: "getData".to_string(),
            new_name: "fetchData".to_string(),
        };

        let (pattern, replacement) = spec.to_pattern_replacement();
        assert!(pattern.contains("getData"));
        assert!(replacement.contains("fetchData"));
    }

    #[test]
    fn test_transform_spec_rename_type() {
        let spec = TransformSpec::RenameType {
            old_name: "UserData".to_string(),
            new_name: "User".to_string(),
        };

        let (pattern, replacement) = spec.to_pattern_replacement();
        assert!(pattern.contains("UserData"));
        assert_eq!(replacement, "User");
    }

    #[test]
    fn test_transform_spec_replace_literal() {
        let spec = TransformSpec::ReplaceLiteral {
            from: "old.api".to_string(),
            to: "new.api".to_string(),
        };

        let (pattern, replacement) = spec.to_pattern_replacement();
        // Should escape the dot
        assert!(pattern.contains(r"\."));
        assert_eq!(replacement, "new.api");
    }

    #[test]
    fn test_upgrade_config_serialization() {
        let mut config = UpgradeConfig::new("test-upgrade", "A test upgrade");
        config.add_transform(TransformSpec::RenameFunction {
            old_name: "old".to_string(),
            new_name: "new".to_string(),
        });

        let yaml = serde_yaml::to_string(&config).unwrap();
        assert!(yaml.contains("test-upgrade"));
        assert!(yaml.contains("rename_function"));

        let parsed: UpgradeConfig = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(parsed.name, "test-upgrade");
        assert_eq!(parsed.transforms.len(), 1);
    }

    #[test]
    fn test_config_based_upgrade_implements_upgrade() {
        let mut config = UpgradeConfig::new("my-upgrade", "My upgrade");
        config.extensions = vec!["ts".to_string()];
        config.add_transform(TransformSpec::ReplaceLiteral {
            from: "old".to_string(),
            to: "new".to_string(),
        });

        let upgrade = config.to_upgrade();

        assert_eq!(upgrade.name(), "my-upgrade");
        assert_eq!(upgrade.description(), "My upgrade");
        assert!(!upgrade.transform().is_empty());
    }

    #[test]
    fn test_upgrade_config_with_versions() {
        let config = UpgradeConfig::new("lib-upgrade", "Upgrade lib")
            .with_versions("v1.0.0", "v2.0.0");

        assert_eq!(config.from_version, Some("v1.0.0".to_string()));
        assert_eq!(config.to_version, Some("v2.0.0".to_string()));
    }

    #[test]
    fn test_upgrade_config_yaml_format() {
        let yaml = r#"
name: my-library-v2
description: Upgrade to my-library v2.0
extensions:
  - ts
  - tsx
transforms:
  - type: rename_function
    old_name: getData
    new_name: fetchData
  - type: replace_literal
    from: "deprecated"
    to: "new_api"
"#;

        let config: UpgradeConfig = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.name, "my-library-v2");
        assert_eq!(config.extensions.len(), 2);
        assert_eq!(config.transforms.len(), 2);
    }
}
