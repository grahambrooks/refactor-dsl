//! Change detection between API versions.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use super::change::{ApiChange, ApiType, ChangeKind, ChangeMetadata, Severity};
use super::signature::{ApiSignature, Parameter};

/// Detects API changes between two versions of a codebase.
pub struct ChangeDetector {
    /// Minimum similarity threshold for rename detection (0.0 - 1.0).
    rename_threshold: f64,
    /// Whether to detect renames across different files.
    cross_file_renames: bool,
    /// Whether to include private API changes.
    include_private: bool,
}

impl Default for ChangeDetector {
    fn default() -> Self {
        Self {
            rename_threshold: 0.7,
            cross_file_renames: true,
            include_private: false,
        }
    }
}

impl ChangeDetector {
    /// Create a new change detector with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the minimum similarity threshold for rename detection.
    pub fn rename_threshold(mut self, threshold: f64) -> Self {
        self.rename_threshold = threshold.clamp(0.0, 1.0);
        self
    }

    /// Enable or disable cross-file rename detection.
    pub fn cross_file_renames(mut self, enabled: bool) -> Self {
        self.cross_file_renames = enabled;
        self
    }

    /// Include private API changes in detection.
    pub fn include_private(mut self, enabled: bool) -> Self {
        self.include_private = enabled;
        self
    }

    /// Detect changes between old and new API signatures.
    pub fn detect(
        &self,
        old_apis: &HashMap<PathBuf, Vec<ApiSignature>>,
        new_apis: &HashMap<PathBuf, Vec<ApiSignature>>,
    ) -> Vec<ApiChange> {
        let mut changes = Vec::new();

        // Build lookup maps
        let old_by_name = self.build_name_map(old_apis);
        let new_by_name = self.build_name_map(new_apis);

        // Track which APIs we've matched
        let mut matched_old: HashSet<String> = HashSet::new();
        let mut matched_new: HashSet<String> = HashSet::new();

        // First pass: exact name matches - detect signature changes
        for (name, old_sig) in &old_by_name {
            if let Some(new_sig) = new_by_name.get(name) {
                matched_old.insert(name.clone());
                matched_new.insert(name.clone());

                // Check for signature changes
                if let Some(change) = self.detect_signature_change(old_sig, new_sig) {
                    changes.push(change);
                }
            }
        }

        // Second pass: detect renames using fuzzy matching
        let unmatched_old: Vec<_> = old_by_name
            .iter()
            .filter(|(name, _)| !matched_old.contains(*name))
            .collect();

        let unmatched_new: Vec<_> = new_by_name
            .iter()
            .filter(|(name, _)| !matched_new.contains(*name))
            .collect();

        let mut rename_pairs = Vec::new();

        for (old_name, old_sig) in &unmatched_old {
            let mut best_match: Option<(&String, &ApiSignature, f64)> = None;

            for (new_name, new_sig) in &unmatched_new {
                // Skip if already paired
                if rename_pairs.iter().any(|(_, n, _)| n == *new_name) {
                    continue;
                }

                // Check if types match
                if old_sig.kind != new_sig.kind {
                    continue;
                }

                // Check cross-file setting
                if !self.cross_file_renames && old_sig.location.file != new_sig.location.file {
                    continue;
                }

                let similarity = self.calculate_similarity(old_sig, new_sig);
                if similarity >= self.rename_threshold
                    && (best_match.is_none() || similarity > best_match.unwrap().2)
                {
                    best_match = Some((new_name, new_sig, similarity));
                }
            }

            if let Some((new_name, new_sig, confidence)) = best_match {
                rename_pairs.push((old_name.to_string(), new_name.to_string(), confidence));
                matched_old.insert(old_name.to_string());
                matched_new.insert(new_name.to_string());

                let change = self.create_rename_change(old_sig, new_sig, confidence);
                changes.push(change);
            }
        }

        // Third pass: detect removed APIs
        for (name, old_sig) in &old_by_name {
            if !matched_old.contains(name) {
                if !self.include_private && !old_sig.is_exported {
                    continue;
                }

                changes.push(ApiChange::new(
                    ChangeKind::ApiRemoved {
                        name: name.clone(),
                        api_type: old_sig.kind,
                    },
                    old_sig.location.file.clone(),
                ).with_original(name.clone())
                .with_metadata(ChangeMetadata::breaking(
                    format!("{} '{}' was removed", old_sig.kind.name(), name)
                )));
            }
        }

        // Sort changes by file path and line number for consistent output
        changes.sort_by(|a, b| {
            a.file_path
                .cmp(&b.file_path)
                .then_with(|| {
                    a.metadata.old_line.cmp(&b.metadata.old_line)
                })
        });

        changes
    }

    fn build_name_map<'a>(
        &self,
        apis: &'a HashMap<PathBuf, Vec<ApiSignature>>,
    ) -> HashMap<String, &'a ApiSignature> {
        let mut map = HashMap::new();

        for signatures in apis.values() {
            for sig in signatures {
                if !self.include_private && !sig.is_exported {
                    continue;
                }

                let key = sig.unique_id();
                map.insert(key, sig);
            }
        }

        map
    }

    fn detect_signature_change(
        &self,
        old_sig: &ApiSignature,
        new_sig: &ApiSignature,
    ) -> Option<ApiChange> {
        // Only check functions/methods
        if !matches!(old_sig.kind, ApiType::Function | ApiType::Method) {
            return None;
        }

        let params_changed = self.params_differ(&old_sig.parameters, &new_sig.parameters);
        let return_changed = old_sig.return_type != new_sig.return_type;

        if !params_changed && !return_changed {
            return None;
        }

        // Check for specific parameter changes
        if let Some(change) = self.detect_parameter_changes(old_sig, new_sig) {
            return Some(change);
        }

        // General signature change
        Some(
            ApiChange::new(
                ChangeKind::SignatureChanged {
                    name: old_sig.name.clone(),
                    old_params: old_sig.parameters.clone(),
                    new_params: new_sig.parameters.clone(),
                    old_return: old_sig.return_type.clone(),
                    new_return: new_sig.return_type.clone(),
                },
                old_sig.location.file.clone(),
            )
            .with_original(format_signature(old_sig))
            .with_replacement(format_signature(new_sig))
            .with_metadata(ChangeMetadata {
                old_line: Some(old_sig.location.line),
                new_line: Some(new_sig.location.line),
                severity: if params_changed {
                    Severity::Breaking
                } else {
                    Severity::Warning
                },
                migration_notes: Some(format!(
                    "Function '{}' signature changed",
                    old_sig.name
                )),
            }),
        )
    }

    fn detect_parameter_changes(
        &self,
        old_sig: &ApiSignature,
        new_sig: &ApiSignature,
    ) -> Option<ApiChange> {
        let old_params = &old_sig.parameters;
        let new_params = &new_sig.parameters;

        // Check for parameter reordering (same names, different order)
        if old_params.len() == new_params.len() {
            let old_names: Vec<_> = old_params.iter().map(|p| &p.name).collect();
            let new_names: Vec<_> = new_params.iter().map(|p| &p.name).collect();

            let old_set: HashSet<_> = old_names.iter().collect();
            let new_set: HashSet<_> = new_names.iter().collect();

            if old_set == new_set && old_names != new_names {
                return Some(
                    ApiChange::new(
                        ChangeKind::ParameterReordered {
                            function_name: old_sig.name.clone(),
                            old_order: old_names.iter().map(|s| (*s).clone()).collect(),
                            new_order: new_names.iter().map(|s| (*s).clone()).collect(),
                        },
                        old_sig.location.file.clone(),
                    )
                    .with_metadata(ChangeMetadata::breaking(format!(
                        "Parameters of '{}' were reordered",
                        old_sig.name
                    ))),
                );
            }
        }

        // Check for added parameters
        let old_names: HashSet<_> = old_params.iter().map(|p| &p.name).collect();
        let new_names: HashSet<_> = new_params.iter().map(|p| &p.name).collect();

        for (pos, param) in new_params.iter().enumerate() {
            if !old_names.contains(&param.name) {
                let severity = if param.has_default || param.is_optional {
                    Severity::Warning
                } else {
                    Severity::Breaking
                };

                return Some(
                    ApiChange::new(
                        ChangeKind::ParameterAdded {
                            function_name: old_sig.name.clone(),
                            param_name: param.name.clone(),
                            param_type: param.type_info.clone(),
                            position: pos,
                            has_default: param.has_default || param.is_optional,
                        },
                        old_sig.location.file.clone(),
                    )
                    .with_metadata(ChangeMetadata {
                        old_line: Some(old_sig.location.line),
                        new_line: Some(new_sig.location.line),
                        severity,
                        migration_notes: Some(format!(
                            "Parameter '{}' added to '{}'",
                            param.name, old_sig.name
                        )),
                    }),
                );
            }
        }

        // Check for removed parameters
        for (pos, param) in old_params.iter().enumerate() {
            if !new_names.contains(&param.name) {
                return Some(
                    ApiChange::new(
                        ChangeKind::ParameterRemoved {
                            function_name: old_sig.name.clone(),
                            param_name: param.name.clone(),
                            position: pos,
                        },
                        old_sig.location.file.clone(),
                    )
                    .with_metadata(ChangeMetadata::breaking(format!(
                        "Parameter '{}' removed from '{}'",
                        param.name, old_sig.name
                    ))),
                );
            }
        }

        None
    }

    fn params_differ(&self, old: &[Parameter], new: &[Parameter]) -> bool {
        if old.len() != new.len() {
            return true;
        }

        for (o, n) in old.iter().zip(new.iter()) {
            if o.name != n.name || o.type_info != n.type_info {
                return true;
            }
        }

        false
    }

    fn calculate_similarity(&self, old: &ApiSignature, new: &ApiSignature) -> f64 {
        let mut score = 0.0;
        let mut weight_sum = 0.0;

        // Name similarity (highest weight)
        let name_sim = string_similarity(&old.name, &new.name);
        score += name_sim * 0.5;
        weight_sum += 0.5;

        // Parameter count similarity
        if matches!(old.kind, ApiType::Function | ApiType::Method) {
            let param_count_sim = if old.parameters.len() == new.parameters.len() {
                1.0
            } else {
                let diff = (old.parameters.len() as i32 - new.parameters.len() as i32).abs();
                1.0 / (1.0 + diff as f64)
            };
            score += param_count_sim * 0.2;
            weight_sum += 0.2;

            // Parameter name overlap
            if !old.parameters.is_empty() || !new.parameters.is_empty() {
                let old_names: HashSet<_> = old.parameters.iter().map(|p| &p.name).collect();
                let new_names: HashSet<_> = new.parameters.iter().map(|p| &p.name).collect();
                let intersection = old_names.intersection(&new_names).count();
                let union = old_names.union(&new_names).count();
                let param_sim = if union > 0 {
                    intersection as f64 / union as f64
                } else {
                    1.0
                };
                score += param_sim * 0.2;
                weight_sum += 0.2;
            }
        }

        // Return type similarity (if present)
        match (&old.return_type, &new.return_type) {
            (Some(old_rt), Some(new_rt)) => {
                let rt_sim = string_similarity(&old_rt.name, &new_rt.name);
                score += rt_sim * 0.1;
                weight_sum += 0.1;
            }
            (None, None) => {
                score += 0.1;
                weight_sum += 0.1;
            }
            _ => {
                weight_sum += 0.1;
            }
        }

        if weight_sum > 0.0 {
            score / weight_sum
        } else {
            0.0
        }
    }

    fn create_rename_change(
        &self,
        old_sig: &ApiSignature,
        new_sig: &ApiSignature,
        confidence: f64,
    ) -> ApiChange {
        let kind = match old_sig.kind {
            ApiType::Function | ApiType::Method => ChangeKind::FunctionRenamed {
                old_name: old_sig.name.clone(),
                new_name: new_sig.name.clone(),
                module_path: old_sig.module_path.clone(),
            },
            ApiType::Class | ApiType::Struct | ApiType::Enum | ApiType::Interface | ApiType::TypeAlias => {
                ChangeKind::TypeRenamed {
                    old_name: old_sig.name.clone(),
                    new_name: new_sig.name.clone(),
                }
            }
            _ => ChangeKind::FunctionRenamed {
                old_name: old_sig.name.clone(),
                new_name: new_sig.name.clone(),
                module_path: old_sig.module_path.clone(),
            },
        };

        ApiChange::new(kind, old_sig.location.file.clone())
            .with_original(&old_sig.name)
            .with_replacement(&new_sig.name)
            .with_confidence(confidence)
            .with_metadata(ChangeMetadata {
                old_line: Some(old_sig.location.line),
                new_line: Some(new_sig.location.line),
                severity: Severity::Breaking,
                migration_notes: Some(format!(
                    "{} '{}' renamed to '{}'",
                    old_sig.kind.name(),
                    old_sig.name,
                    new_sig.name
                )),
            })
    }
}

/// Calculate string similarity using Levenshtein distance.
fn string_similarity(a: &str, b: &str) -> f64 {
    if a == b {
        return 1.0;
    }

    let len_a = a.chars().count();
    let len_b = b.chars().count();

    if len_a == 0 || len_b == 0 {
        return 0.0;
    }

    let distance = levenshtein_distance(a, b);
    let max_len = len_a.max(len_b);

    1.0 - (distance as f64 / max_len as f64)
}

/// Calculate Levenshtein edit distance between two strings.
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let len_a = a_chars.len();
    let len_b = b_chars.len();

    if len_a == 0 {
        return len_b;
    }
    if len_b == 0 {
        return len_a;
    }

    let mut matrix = vec![vec![0usize; len_b + 1]; len_a + 1];

    for (i, row) in matrix.iter_mut().enumerate() {
        row[0] = i;
    }
    for (j, val) in matrix[0].iter_mut().enumerate() {
        *val = j;
    }

    for i in 1..=len_a {
        for j in 1..=len_b {
            let cost = if a_chars[i - 1] == b_chars[j - 1] {
                0
            } else {
                1
            };

            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[len_a][len_b]
}

fn format_signature(sig: &ApiSignature) -> String {
    let params: Vec<String> = sig.parameters.iter().map(|p| p.display()).collect();
    let params_str = params.join(", ");

    let ret = sig
        .return_type
        .as_ref()
        .map(|t| format!(" -> {}", t.display()))
        .unwrap_or_default();

    format!("{}({}){}", sig.name, params_str, ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::signature::{SourceLocation, Visibility};

    fn make_fn(name: &str, params: Vec<&str>) -> ApiSignature {
        let loc = SourceLocation::new("test.rs", 1, 1);
        let params = params
            .into_iter()
            .map(|n| Parameter::new(n))
            .collect();

        ApiSignature::function(name, loc)
            .with_params(params)
            .with_visibility(Visibility::Public)
            .exported(true)
    }

    #[test]
    fn test_detect_removed_api() {
        let detector = ChangeDetector::new();

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("foo", vec!["x"]), make_fn("bar", vec!["y"])],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(PathBuf::from("lib.rs"), vec![make_fn("foo", vec!["x"])]);

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::ApiRemoved { name, .. } if name == "bar"
        ));
    }

    #[test]
    fn test_detect_rename() {
        let detector = ChangeDetector::new().rename_threshold(0.5);

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("getUserById", vec!["id"])],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("fetchUserById", vec!["id"])],
        );

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::FunctionRenamed { old_name, new_name, .. }
            if old_name == "getUserById" && new_name == "fetchUserById"
        ));
    }

    #[test]
    fn test_detect_parameter_added() {
        let detector = ChangeDetector::new();

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("process", vec!["data"])],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("process", vec!["data", "options"])],
        );

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::ParameterAdded { function_name, param_name, .. }
            if function_name == "process" && param_name == "options"
        ));
    }

    #[test]
    fn test_detect_parameter_removed() {
        let detector = ChangeDetector::new();

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("process", vec!["data", "debug"])],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("process", vec!["data"])],
        );

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::ParameterRemoved { function_name, param_name, .. }
            if function_name == "process" && param_name == "debug"
        ));
    }

    #[test]
    fn test_detect_parameter_reordered() {
        let detector = ChangeDetector::new();

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("draw", vec!["x", "y", "color"])],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("draw", vec!["color", "x", "y"])],
        );

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::ParameterReordered { function_name, .. }
            if function_name == "draw"
        ));
    }

    #[test]
    fn test_levenshtein_distance() {
        assert_eq!(levenshtein_distance("", ""), 0);
        assert_eq!(levenshtein_distance("abc", "abc"), 0);
        assert_eq!(levenshtein_distance("abc", ""), 3);
        assert_eq!(levenshtein_distance("", "abc"), 3);
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("saturday", "sunday"), 3);
    }

    #[test]
    fn test_string_similarity() {
        assert!((string_similarity("hello", "hello") - 1.0).abs() < f64::EPSILON);
        assert!((string_similarity("hello", "hallo") - 0.8).abs() < 0.01);
        assert!(string_similarity("abc", "xyz") < 0.5);
    }

    #[test]
    fn test_no_changes() {
        let detector = ChangeDetector::new();

        let mut apis = HashMap::new();
        apis.insert(
            PathBuf::from("lib.rs"),
            vec![make_fn("foo", vec!["x"]), make_fn("bar", vec!["y"])],
        );

        let changes = detector.detect(&apis, &apis);
        assert!(changes.is_empty());
    }

    #[test]
    fn test_type_rename() {
        // Use a lower threshold since "UserData" -> "UserInfo" has ~0.5 similarity
        let detector = ChangeDetector::new().rename_threshold(0.4);

        let loc = SourceLocation::new("types.rs", 1, 1);

        let mut old_apis = HashMap::new();
        old_apis.insert(
            PathBuf::from("types.rs"),
            vec![ApiSignature::type_def("UserData", ApiType::Struct, loc.clone())
                .with_visibility(Visibility::Public)
                .exported(true)],
        );

        let mut new_apis = HashMap::new();
        new_apis.insert(
            PathBuf::from("types.rs"),
            vec![ApiSignature::type_def("UserInfo", ApiType::Struct, loc)
                .with_visibility(Visibility::Public)
                .exported(true)],
        );

        let changes = detector.detect(&old_apis, &new_apis);

        assert_eq!(changes.len(), 1);
        assert!(matches!(
            &changes[0].kind,
            ChangeKind::TypeRenamed { old_name, new_name }
            if old_name == "UserData" && new_name == "UserInfo"
        ));
    }
}
