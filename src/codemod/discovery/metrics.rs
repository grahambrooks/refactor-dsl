//! Code metrics filtering.

use std::path::Path;
use std::time::{Duration, SystemTime};

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Comparison operators for metric filtering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComparisonOp {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Greater than or equal to
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal to
    LessThanOrEqual,
    /// Between (inclusive)
    Between,
}

impl ComparisonOp {
    /// Compare two values.
    pub fn compare(&self, value: f64, target: f64, target2: Option<f64>) -> bool {
        match self {
            Self::Equals => (value - target).abs() < f64::EPSILON,
            Self::NotEquals => (value - target).abs() >= f64::EPSILON,
            Self::GreaterThan => value > target,
            Self::GreaterThanOrEqual => value >= target,
            Self::LessThan => value < target,
            Self::LessThanOrEqual => value <= target,
            Self::Between => {
                if let Some(t2) = target2 {
                    value >= target && value <= t2
                } else {
                    value >= target
                }
            }
        }
    }
}

/// A metric condition.
#[derive(Debug, Clone)]
pub struct MetricCondition {
    /// Comparison operator.
    pub op: ComparisonOp,
    /// Target value.
    pub target: f64,
    /// Second target value (for between).
    pub target2: Option<f64>,
}

impl MetricCondition {
    /// Create an equals condition.
    pub fn equals(value: f64) -> Self {
        Self {
            op: ComparisonOp::Equals,
            target: value,
            target2: None,
        }
    }

    /// Create a greater than condition.
    pub fn greater_than(value: f64) -> Self {
        Self {
            op: ComparisonOp::GreaterThan,
            target: value,
            target2: None,
        }
    }

    /// Create a less than condition.
    pub fn less_than(value: f64) -> Self {
        Self {
            op: ComparisonOp::LessThan,
            target: value,
            target2: None,
        }
    }

    /// Create a between condition.
    pub fn between(min: f64, max: f64) -> Self {
        Self {
            op: ComparisonOp::Between,
            target: min,
            target2: Some(max),
        }
    }

    /// Check if a value satisfies this condition.
    pub fn check(&self, value: f64) -> bool {
        self.op.compare(value, self.target, self.target2)
    }
}

/// Filter repositories by code metrics.
#[derive(Debug, Clone, Default)]
pub struct MetricFilter {
    /// Lines of code condition.
    pub lines_of_code: Option<MetricCondition>,
    /// File count condition.
    pub file_count: Option<MetricCondition>,
    /// Directory count condition.
    pub directory_count: Option<MetricCondition>,
    /// Last commit age in days condition.
    pub commit_age_days: Option<MetricCondition>,
    /// Repository age in days condition.
    pub repo_age_days: Option<MetricCondition>,
    /// Average file size in bytes condition.
    pub avg_file_size: Option<MetricCondition>,
    /// Maximum file size in bytes condition.
    pub max_file_size: Option<MetricCondition>,
    /// Test coverage percentage condition (if available).
    pub test_coverage: Option<MetricCondition>,
    /// Number of contributors condition.
    pub contributor_count: Option<MetricCondition>,
    /// Commit count condition.
    pub commit_count: Option<MetricCondition>,
}

impl MetricFilter {
    /// Create a new metric filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Filter by lines of code.
    pub fn lines_of_code(mut self, op: ComparisonOp, value: f64) -> Self {
        self.lines_of_code = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by lines of code between values.
    pub fn lines_of_code_between(mut self, min: f64, max: f64) -> Self {
        self.lines_of_code = Some(MetricCondition::between(min, max));
        self
    }

    /// Filter by file count.
    pub fn file_count(mut self, op: ComparisonOp, value: f64) -> Self {
        self.file_count = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by directory count.
    pub fn directory_count(mut self, op: ComparisonOp, value: f64) -> Self {
        self.directory_count = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by last commit age in days.
    pub fn commit_age_days(mut self, op: ComparisonOp, value: f64) -> Self {
        self.commit_age_days = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by average file size.
    pub fn avg_file_size(mut self, op: ComparisonOp, value: f64) -> Self {
        self.avg_file_size = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by maximum file size.
    pub fn max_file_size(mut self, op: ComparisonOp, value: f64) -> Self {
        self.max_file_size = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by contributor count.
    pub fn contributor_count(mut self, op: ComparisonOp, value: f64) -> Self {
        self.contributor_count = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Filter by commit count.
    pub fn commit_count(mut self, op: ComparisonOp, value: f64) -> Self {
        self.commit_count = Some(MetricCondition {
            op,
            target: value,
            target2: None,
        });
        self
    }

    /// Check if a repository matches this filter.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        let metrics = RepositoryMetrics::analyze(repo_path)?;

        if let Some(ref cond) = self.lines_of_code {
            if !cond.check(metrics.lines_of_code as f64) {
                return Ok(false);
            }
        }

        if let Some(ref cond) = self.file_count {
            if !cond.check(metrics.file_count as f64) {
                return Ok(false);
            }
        }

        if let Some(ref cond) = self.directory_count {
            if !cond.check(metrics.directory_count as f64) {
                return Ok(false);
            }
        }

        if let Some(ref cond) = self.commit_age_days {
            if let Some(age) = metrics.last_commit_age_days {
                if !cond.check(age) {
                    return Ok(false);
                }
            }
        }

        if let Some(ref cond) = self.avg_file_size {
            if !cond.check(metrics.avg_file_size) {
                return Ok(false);
            }
        }

        if let Some(ref cond) = self.max_file_size {
            if !cond.check(metrics.max_file_size as f64) {
                return Ok(false);
            }
        }

        if let Some(ref cond) = self.contributor_count {
            if let Some(count) = metrics.contributor_count {
                if !cond.check(count as f64) {
                    return Ok(false);
                }
            }
        }

        if let Some(ref cond) = self.commit_count {
            if let Some(count) = metrics.commit_count {
                if !cond.check(count as f64) {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }
}

/// Repository metrics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RepositoryMetrics {
    /// Total lines of code.
    pub lines_of_code: usize,
    /// Number of files.
    pub file_count: usize,
    /// Number of directories.
    pub directory_count: usize,
    /// Lines of code by extension.
    pub lines_by_extension: std::collections::HashMap<String, usize>,
    /// Average file size in bytes.
    pub avg_file_size: f64,
    /// Maximum file size in bytes.
    pub max_file_size: usize,
    /// Total size in bytes.
    pub total_size: usize,
    /// Age of last commit in days.
    pub last_commit_age_days: Option<f64>,
    /// Repository age in days.
    pub repo_age_days: Option<f64>,
    /// Number of contributors.
    pub contributor_count: Option<usize>,
    /// Total commit count.
    pub commit_count: Option<usize>,
}

impl RepositoryMetrics {
    /// Analyze a repository and collect metrics.
    pub fn analyze(repo_path: &Path) -> Result<Self> {
        let mut metrics = Self::default();
        let mut lines_by_ext: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        // Walk the repository
        Self::walk_directory(repo_path, &mut metrics, &mut lines_by_ext)?;

        metrics.lines_by_extension = lines_by_ext;

        // Calculate averages
        if metrics.file_count > 0 {
            metrics.avg_file_size = metrics.total_size as f64 / metrics.file_count as f64;
        }

        // Try to get git metrics
        metrics.collect_git_metrics(repo_path);

        Ok(metrics)
    }

    /// Walk a directory and collect file metrics.
    fn walk_directory(
        dir: &Path,
        metrics: &mut RepositoryMetrics,
        lines_by_ext: &mut std::collections::HashMap<String, usize>,
    ) -> Result<()> {
        let entries = match std::fs::read_dir(dir) {
            Ok(entries) => entries,
            Err(_) => return Ok(()),
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Skip hidden files and common non-source directories
            if name.starts_with('.') || Self::should_skip(name) {
                continue;
            }

            if path.is_dir() {
                metrics.directory_count += 1;
                Self::walk_directory(&path, metrics, lines_by_ext)?;
            } else if path.is_file() {
                metrics.file_count += 1;

                if let Ok(metadata) = path.metadata() {
                    let size = metadata.len() as usize;
                    metrics.total_size += size;
                    metrics.max_file_size = metrics.max_file_size.max(size);
                }

                // Count lines for source files
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if Self::is_source_extension(ext) {
                        if let Ok(content) = std::fs::read_to_string(&path) {
                            let line_count = content.lines().count();
                            metrics.lines_of_code += line_count;
                            *lines_by_ext.entry(ext.to_string()).or_insert(0) += line_count;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a directory should be skipped.
    fn should_skip(name: &str) -> bool {
        matches!(
            name,
            "node_modules"
                | "target"
                | "build"
                | "dist"
                | "vendor"
                | "__pycache__"
                | ".git"
                | ".svn"
                | ".hg"
                | "venv"
                | ".venv"
                | "env"
                | ".env"
        )
    }

    /// Check if a file extension is for source code.
    fn is_source_extension(ext: &str) -> bool {
        matches!(
            ext.to_lowercase().as_str(),
            "rs" | "py" | "js" | "ts" | "jsx" | "tsx" | "go" | "java" | "kt" | "scala"
                | "rb" | "php" | "c" | "cpp" | "cc" | "cxx" | "h" | "hpp"
                | "cs" | "fs" | "swift" | "m" | "mm" | "vue" | "svelte"
                | "lua" | "pl" | "pm" | "r" | "jl" | "hs" | "elm" | "ex" | "exs"
                | "erl" | "clj" | "cljs" | "lisp" | "scm" | "ml" | "mli"
        )
    }

    /// Collect git-related metrics.
    fn collect_git_metrics(&mut self, repo_path: &Path) {
        // Check if this is a git repository
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return;
        }

        // Try to get commit count
        if let Ok(output) = std::process::Command::new("git")
            .args(["rev-list", "--count", "HEAD"])
            .current_dir(repo_path)
            .output()
        {
            if output.status.success() {
                if let Ok(count_str) = String::from_utf8(output.stdout) {
                    if let Ok(count) = count_str.trim().parse::<usize>() {
                        self.commit_count = Some(count);
                    }
                }
            }
        }

        // Try to get contributor count
        if let Ok(output) = std::process::Command::new("git")
            .args(["shortlog", "-sn", "HEAD"])
            .current_dir(repo_path)
            .output()
        {
            if output.status.success() {
                if let Ok(log) = String::from_utf8(output.stdout) {
                    self.contributor_count = Some(log.lines().count());
                }
            }
        }

        // Try to get last commit date
        if let Ok(output) = std::process::Command::new("git")
            .args(["log", "-1", "--format=%ct"])
            .current_dir(repo_path)
            .output()
        {
            if output.status.success() {
                if let Ok(timestamp_str) = String::from_utf8(output.stdout) {
                    if let Ok(timestamp) = timestamp_str.trim().parse::<u64>() {
                        let commit_time = std::time::UNIX_EPOCH + Duration::from_secs(timestamp);
                        if let Ok(age) = SystemTime::now().duration_since(commit_time) {
                            self.last_commit_age_days = Some(age.as_secs_f64() / 86400.0);
                        }
                    }
                }
            }
        }

        // Try to get first commit date (repo age)
        if let Ok(output) = std::process::Command::new("git")
            .args(["log", "--reverse", "--format=%ct", "-1"])
            .current_dir(repo_path)
            .output()
        {
            if output.status.success() {
                if let Ok(timestamp_str) = String::from_utf8(output.stdout) {
                    if let Ok(timestamp) = timestamp_str.trim().parse::<u64>() {
                        let first_commit_time = std::time::UNIX_EPOCH + Duration::from_secs(timestamp);
                        if let Ok(age) = SystemTime::now().duration_since(first_commit_time) {
                            self.repo_age_days = Some(age.as_secs_f64() / 86400.0);
                        }
                    }
                }
            }
        }
    }

    /// Get the primary language by lines of code.
    pub fn primary_language(&self) -> Option<&str> {
        self.lines_by_extension
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(ext, _)| ext.as_str())
    }

    /// Get language distribution as percentages.
    pub fn language_distribution(&self) -> std::collections::HashMap<String, f64> {
        let total = self.lines_of_code as f64;
        if total == 0.0 {
            return std::collections::HashMap::new();
        }

        self.lines_by_extension
            .iter()
            .map(|(ext, count)| (ext.clone(), *count as f64 / total * 100.0))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comparison_op() {
        assert!(ComparisonOp::Equals.compare(1.0, 1.0, None));
        assert!(!ComparisonOp::Equals.compare(1.0, 2.0, None));

        assert!(ComparisonOp::GreaterThan.compare(2.0, 1.0, None));
        assert!(!ComparisonOp::GreaterThan.compare(1.0, 2.0, None));

        assert!(ComparisonOp::LessThan.compare(1.0, 2.0, None));
        assert!(!ComparisonOp::LessThan.compare(2.0, 1.0, None));

        assert!(ComparisonOp::Between.compare(5.0, 1.0, Some(10.0)));
        assert!(!ComparisonOp::Between.compare(15.0, 1.0, Some(10.0)));
    }

    #[test]
    fn test_metric_condition() {
        let cond = MetricCondition::greater_than(100.0);
        assert!(cond.check(150.0));
        assert!(!cond.check(50.0));

        let cond = MetricCondition::between(10.0, 20.0);
        assert!(cond.check(15.0));
        assert!(!cond.check(25.0));
    }

    #[test]
    fn test_metric_filter_builder() {
        let filter = MetricFilter::new()
            .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
            .file_count(ComparisonOp::LessThan, 100.0)
            .commit_age_days(ComparisonOp::LessThan, 30.0);

        assert!(filter.lines_of_code.is_some());
        assert!(filter.file_count.is_some());
        assert!(filter.commit_age_days.is_some());
    }

    #[test]
    fn test_is_source_extension() {
        assert!(RepositoryMetrics::is_source_extension("rs"));
        assert!(RepositoryMetrics::is_source_extension("py"));
        assert!(RepositoryMetrics::is_source_extension("js"));
        assert!(!RepositoryMetrics::is_source_extension("txt"));
        assert!(!RepositoryMetrics::is_source_extension("md"));
    }

    #[test]
    fn test_should_skip() {
        assert!(RepositoryMetrics::should_skip("node_modules"));
        assert!(RepositoryMetrics::should_skip("target"));
        assert!(!RepositoryMetrics::should_skip("src"));
        assert!(!RepositoryMetrics::should_skip("lib"));
    }
}
