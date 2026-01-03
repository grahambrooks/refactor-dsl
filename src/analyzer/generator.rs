//! Transform generation from detected API changes.

use std::fmt::Write;

use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::codemod::Upgrade;
use crate::matcher::Matcher;
use crate::transform::TransformBuilder;

use super::change::{ApiChange, ChangeKind, Severity};

/// Generates transforms from detected API changes.
pub struct UpgradeGenerator {
    /// The name for the generated upgrade.
    name: String,
    /// Description for the generated upgrade.
    description: String,
    /// Detected changes to generate transforms from.
    changes: Vec<ApiChange>,
    /// File extensions to target.
    extensions: Vec<String>,
}

impl UpgradeGenerator {
    /// Create a new upgrade generator.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            changes: Vec::new(),
            extensions: vec!["rs".to_string(), "ts".to_string(), "py".to_string()],
        }
    }

    /// Add changes to generate transforms from.
    pub fn with_changes(mut self, changes: Vec<ApiChange>) -> Self {
        self.changes = changes;
        self
    }

    /// Set target file extensions.
    pub fn for_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Generate an upgrade that can be applied to dependent projects.
    pub fn generate(self) -> GeneratedUpgrade {
        let mut transforms = Vec::new();

        for change in &self.changes {
            if let Some(transform) = self.change_to_transform(change) {
                transforms.push(transform);
            }
        }

        GeneratedUpgrade {
            name: self.name,
            description: self.description,
            changes: self.changes,
            transforms,
            extensions: self.extensions,
        }
    }

    fn change_to_transform(&self, change: &ApiChange) -> Option<Transform> {
        match &change.kind {
            ChangeKind::FunctionRenamed {
                old_name,
                new_name,
                module_path: _,
            } => Some(Transform::FunctionRename {
                old_name: old_name.clone(),
                new_name: new_name.clone(),
            }),

            ChangeKind::TypeRenamed { old_name, new_name } => Some(Transform::TypeRename {
                old_name: old_name.clone(),
                new_name: new_name.clone(),
            }),

            ChangeKind::ImportRenamed { old_path, new_path } => Some(Transform::ImportRename {
                old_path: old_path.clone(),
                new_path: new_path.clone(),
            }),

            ChangeKind::MethodMoved {
                method_name,
                old_location,
                new_location,
            } => Some(Transform::MethodMove {
                method_name: method_name.clone(),
                old_location: old_location.clone(),
                new_location: new_location.clone(),
            }),

            ChangeKind::ConstantChanged {
                name,
                old_value,
                new_value,
            } => {
                if let (Some(old), Some(new)) = (old_value, new_value) {
                    Some(Transform::ConstantUpdate {
                        name: name.clone(),
                        old_value: old.clone(),
                        new_value: new.clone(),
                    })
                } else {
                    None
                }
            }

            // These changes cannot be auto-transformed
            ChangeKind::SignatureChanged { .. }
            | ChangeKind::ParameterAdded { .. }
            | ChangeKind::ParameterRemoved { .. }
            | ChangeKind::ParameterReordered { .. }
            | ChangeKind::ApiRemoved { .. }
            | ChangeKind::TypeChanged { .. } => None,
        }
    }
}

/// A transform to apply for an API change.
#[derive(Debug, Clone)]
pub enum Transform {
    FunctionRename {
        old_name: String,
        new_name: String,
    },
    TypeRename {
        old_name: String,
        new_name: String,
    },
    ImportRename {
        old_path: String,
        new_path: String,
    },
    MethodMove {
        method_name: String,
        old_location: String,
        new_location: String,
    },
    ConstantUpdate {
        name: String,
        old_value: String,
        new_value: String,
    },
}

impl Transform {
    /// Convert this transform to a regex pattern and replacement.
    pub fn to_pattern_replacement(&self) -> (String, String) {
        match self {
            Transform::FunctionRename { old_name, new_name } => {
                // Match function call: old_name( or old_name::<...>(
                let pattern = format!(r"\b{}\s*(\(|::<)", regex::escape(old_name));
                let replacement = format!("{}$1", new_name);
                (pattern, replacement)
            }

            Transform::TypeRename { old_name, new_name } => {
                // Match type usage: word boundaries around the type name
                let pattern = format!(r"\b{}\b", regex::escape(old_name));
                (pattern, new_name.clone())
            }

            Transform::ImportRename { old_path, new_path } => {
                // Match import paths in various syntaxes
                let pattern = format!(r#"(['"]){}\1"#, regex::escape(old_path));
                let replacement = format!("$1{}$1", new_path);
                (pattern, replacement)
            }

            Transform::MethodMove {
                method_name,
                old_location,
                new_location,
            } => {
                // Match method call on old location
                let pattern = format!(
                    r"{}\.{}",
                    regex::escape(old_location),
                    regex::escape(method_name)
                );
                let replacement = format!("{}.{}", new_location, method_name);
                (pattern, replacement)
            }

            Transform::ConstantUpdate {
                name: _,
                old_value,
                new_value,
            } => {
                // Literal replacement of values
                (regex::escape(old_value), new_value.clone())
            }
        }
    }
}

/// A generated upgrade that implements the Upgrade trait.
#[derive(Debug, Clone)]
pub struct GeneratedUpgrade {
    /// Name of the upgrade.
    pub name: String,
    /// Description of the upgrade.
    pub description: String,
    /// The detected changes this upgrade addresses.
    pub changes: Vec<ApiChange>,
    /// The transforms to apply.
    pub transforms: Vec<Transform>,
    /// File extensions to match.
    pub extensions: Vec<String>,
}

impl GeneratedUpgrade {
    /// Get changes that require manual review.
    pub fn manual_review_changes(&self) -> Vec<&ApiChange> {
        self.changes
            .iter()
            .filter(|c| c.requires_manual_review())
            .collect()
    }

    /// Get breaking changes.
    pub fn breaking_changes(&self) -> Vec<&ApiChange> {
        self.changes.iter().filter(|c| c.is_breaking()).collect()
    }

    /// Get auto-transformable changes.
    pub fn auto_transformable_changes(&self) -> Vec<&ApiChange> {
        self.changes
            .iter()
            .filter(|c| c.kind.is_auto_transformable())
            .collect()
    }

    /// Generate a human-readable report of the changes.
    pub fn report(&self) -> String {
        let mut report = String::new();

        writeln!(report, "# Upgrade Report: {}", self.name).unwrap();
        writeln!(report).unwrap();
        writeln!(report, "{}", self.description).unwrap();
        writeln!(report).unwrap();

        // Summary
        let breaking = self.breaking_changes().len();
        let auto = self.auto_transformable_changes().len();
        let manual = self.manual_review_changes().len();

        writeln!(report, "## Summary").unwrap();
        writeln!(report).unwrap();
        writeln!(report, "- Total changes: {}", self.changes.len()).unwrap();
        writeln!(report, "- Auto-transformable: {}", auto).unwrap();
        writeln!(report, "- Breaking changes: {}", breaking).unwrap();
        writeln!(report, "- Requires manual review: {}", manual).unwrap();
        writeln!(report).unwrap();

        // Auto-transformable changes
        let auto_changes = self.auto_transformable_changes();
        if !auto_changes.is_empty() {
            writeln!(report, "## Automatic Transforms").unwrap();
            writeln!(report).unwrap();
            for change in auto_changes {
                writeln!(report, "- {}", format_change(change)).unwrap();
            }
            writeln!(report).unwrap();
        }

        // Manual review changes
        let manual_changes = self.manual_review_changes();
        if !manual_changes.is_empty() {
            writeln!(report, "## Requires Manual Review").unwrap();
            writeln!(report).unwrap();
            for change in manual_changes {
                writeln!(report, "- {}", format_change(change)).unwrap();
                if let Some(ref notes) = change.metadata.migration_notes {
                    writeln!(report, "  - {}", notes).unwrap();
                }
            }
            writeln!(report).unwrap();
        }

        report
    }

    /// Generate Rust source code for a static upgrade.
    pub fn to_rust_source(&self, struct_name: &str) -> String {
        let tokens = self.to_rust_tokens(struct_name);
        format_rust_code(tokens)
    }

    /// Generate Rust tokens for a static upgrade.
    fn to_rust_tokens(&self, struct_name: &str) -> TokenStream {
        let struct_ident = format_ident!("{}", struct_name);
        let name = &self.name;
        let description = &self.description;

        let matcher_body = self.generate_matcher_tokens();
        let transform_body = self.generate_transform_tokens();

        quote! {
            use refactor::codemod::Upgrade;
            use refactor::matcher::Matcher;
            use refactor::transform::TransformBuilder;

            #[doc = #description]
            pub struct #struct_ident;

            impl Upgrade for #struct_ident {
                fn name(&self) -> &str {
                    #name
                }

                fn description(&self) -> &str {
                    #description
                }

                fn matcher(&self) -> Matcher {
                    #matcher_body
                }

                fn transform(&self) -> TransformBuilder {
                    #transform_body
                }
            }
        }
    }

    fn generate_matcher_tokens(&self) -> TokenStream {
        if self.extensions.is_empty() {
            quote! { Matcher::new() }
        } else {
            let exts = &self.extensions;
            quote! {
                Matcher::new().files(|f| {
                    f.extensions([#(#exts),*])
                        .exclude("**/node_modules/**")
                })
            }
        }
    }

    fn generate_transform_tokens(&self) -> TokenStream {
        let replace_calls: Vec<TokenStream> = self
            .transforms
            .iter()
            .map(|t| {
                let (pattern, replacement) = t.to_pattern_replacement();
                quote! {
                    .replace_pattern(#pattern, #replacement)
                }
            })
            .collect();

        quote! {
            TransformBuilder::new()
                #(#replace_calls)*
        }
    }
}

/// Format Rust tokens into a pretty-printed string.
fn format_rust_code(tokens: TokenStream) -> String {
    let syntax_tree = syn::parse_file(&tokens.to_string()).expect("Failed to parse generated code");
    prettyplease::unparse(&syntax_tree)
}

impl Upgrade for GeneratedUpgrade {
    fn name(&self) -> &str {
        &self.name
    }

    fn description(&self) -> &str {
        &self.description
    }

    fn matcher(&self) -> Matcher {
        if self.extensions.is_empty() {
            Matcher::new()
        } else {
            // Convert String to &str for extensions
            let extensions: Vec<&str> = self.extensions.iter().map(|s| s.as_str()).collect();
            Matcher::new().files(move |f| {
                f.extensions(extensions.clone())
                    .exclude("**/node_modules/**")
                    .exclude("**/target/**")
                    .exclude("**/.git/**")
            })
        }
    }

    fn transform(&self) -> TransformBuilder {
        let mut builder = TransformBuilder::new();

        for transform in &self.transforms {
            let (pattern, replacement) = transform.to_pattern_replacement();
            builder = builder.replace_pattern(&pattern, &replacement);
        }

        builder
    }
}

fn format_change(change: &ApiChange) -> String {
    let severity_icon = match change.metadata.severity {
        Severity::Breaking => "!!",
        Severity::Warning => "! ",
        Severity::Info => "  ",
    };

    let confidence = if change.confidence < 1.0 {
        format!(" (confidence: {:.0}%)", change.confidence * 100.0)
    } else {
        String::new()
    };

    format!(
        "[{}] {}: {}{}",
        severity_icon,
        change.kind.name(),
        format_change_detail(&change.kind),
        confidence
    )
}

fn format_change_detail(kind: &ChangeKind) -> String {
    match kind {
        ChangeKind::FunctionRenamed {
            old_name, new_name, ..
        } => {
            format!("`{}` -> `{}`", old_name, new_name)
        }
        ChangeKind::TypeRenamed { old_name, new_name } => {
            format!("`{}` -> `{}`", old_name, new_name)
        }
        ChangeKind::ImportRenamed { old_path, new_path } => {
            format!("`{}` -> `{}`", old_path, new_path)
        }
        ChangeKind::SignatureChanged { name, .. } => {
            format!("`{}` signature changed", name)
        }
        ChangeKind::ParameterAdded {
            function_name,
            param_name,
            ..
        } => {
            format!("`{}`: added `{}`", function_name, param_name)
        }
        ChangeKind::ParameterRemoved {
            function_name,
            param_name,
            ..
        } => {
            format!("`{}`: removed `{}`", function_name, param_name)
        }
        ChangeKind::ParameterReordered { function_name, .. } => {
            format!("`{}`: parameters reordered", function_name)
        }
        ChangeKind::ApiRemoved { name, api_type } => {
            format!("{} `{}` removed", api_type.name(), name)
        }
        ChangeKind::TypeChanged { name, description } => {
            format!("`{}`: {}", name, description)
        }
        ChangeKind::MethodMoved {
            method_name,
            old_location,
            new_location,
        } => {
            format!(
                "`{}`: `{}` -> `{}`",
                method_name, old_location, new_location
            )
        }
        ChangeKind::ConstantChanged {
            name,
            old_value,
            new_value,
        } => {
            let old = old_value.as_deref().unwrap_or("?");
            let new = new_value.as_deref().unwrap_or("?");
            format!("`{}`: `{}` -> `{}`", name, old, new)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::ApiType;
    use std::path::PathBuf;

    #[test]
    fn test_function_rename_transform() {
        let change = ApiChange::new(
            ChangeKind::FunctionRenamed {
                old_name: "getUserById".into(),
                new_name: "fetchUserById".into(),
                module_path: None,
            },
            PathBuf::from("lib.rs"),
        );

        let upgrade = UpgradeGenerator::new("test", "test upgrade")
            .with_changes(vec![change])
            .generate();

        assert_eq!(upgrade.transforms.len(), 1);
        assert!(matches!(
            &upgrade.transforms[0],
            Transform::FunctionRename { old_name, new_name }
            if old_name == "getUserById" && new_name == "fetchUserById"
        ));
    }

    #[test]
    fn test_type_rename_transform() {
        let change = ApiChange::new(
            ChangeKind::TypeRenamed {
                old_name: "UserData".into(),
                new_name: "UserInfo".into(),
            },
            PathBuf::from("types.rs"),
        );

        let upgrade = UpgradeGenerator::new("test", "test upgrade")
            .with_changes(vec![change])
            .generate();

        assert_eq!(upgrade.transforms.len(), 1);

        let (pattern, replacement) = upgrade.transforms[0].to_pattern_replacement();
        assert!(pattern.contains("UserData"));
        assert_eq!(replacement, "UserInfo");
    }

    #[test]
    fn test_non_transformable_changes() {
        let change = ApiChange::new(
            ChangeKind::ApiRemoved {
                name: "deprecated_fn".into(),
                api_type: ApiType::Function,
            },
            PathBuf::from("lib.rs"),
        );

        let upgrade = UpgradeGenerator::new("test", "test upgrade")
            .with_changes(vec![change])
            .generate();

        assert!(upgrade.transforms.is_empty());
        assert_eq!(upgrade.manual_review_changes().len(), 1);
    }

    #[test]
    fn test_report_generation() {
        let changes = vec![
            ApiChange::new(
                ChangeKind::FunctionRenamed {
                    old_name: "old_fn".into(),
                    new_name: "new_fn".into(),
                    module_path: None,
                },
                PathBuf::from("lib.rs"),
            ),
            ApiChange::new(
                ChangeKind::ApiRemoved {
                    name: "removed_fn".into(),
                    api_type: ApiType::Function,
                },
                PathBuf::from("lib.rs"),
            ),
        ];

        let upgrade = UpgradeGenerator::new("my-upgrade", "My library upgrade")
            .with_changes(changes)
            .generate();

        let report = upgrade.report();
        assert!(report.contains("my-upgrade"));
        assert!(report.contains("old_fn"));
        assert!(report.contains("new_fn"));
        assert!(report.contains("removed_fn"));
        assert!(report.contains("Manual Review"));
    }

    #[test]
    fn test_rust_source_generation() {
        let changes = vec![ApiChange::new(
            ChangeKind::FunctionRenamed {
                old_name: "getData".into(),
                new_name: "fetchData".into(),
                module_path: None,
            },
            PathBuf::from("lib.rs"),
        )];

        let upgrade = UpgradeGenerator::new("data-upgrade", "Update data fetching API")
            .with_changes(changes)
            .for_extensions(vec!["ts".to_string()])
            .generate();

        let source = upgrade.to_rust_source("DataUpgrade");
        assert!(source.contains("pub struct DataUpgrade"));
        assert!(source.contains("impl Upgrade for DataUpgrade"));
        assert!(source.contains("getData"));
        assert!(source.contains("fetchData"));
    }

    #[test]
    fn test_generated_upgrade_implements_upgrade_trait() {
        let changes = vec![ApiChange::new(
            ChangeKind::TypeRenamed {
                old_name: "OldType".into(),
                new_name: "NewType".into(),
            },
            PathBuf::from("types.rs"),
        )];

        let upgrade = UpgradeGenerator::new("type-upgrade", "Rename types")
            .with_changes(changes)
            .for_extensions(vec!["rs".to_string()])
            .generate();

        // Test that it implements Upgrade
        assert_eq!(upgrade.name(), "type-upgrade");
        assert_eq!(upgrade.description(), "Rename types");

        let transform = upgrade.transform();
        assert!(!transform.is_empty());
    }
}
