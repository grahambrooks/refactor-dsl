//! Advanced repository discovery and filtering.
//!
//! This module provides sophisticated filtering capabilities for repositories,
//! allowing you to filter by dependencies, frameworks, code metrics, and languages.
//!
//! ## Example
//!
//! ```rust,no_run
//! use refactor::codemod::discovery::{
//!     AdvancedRepoFilter, DependencyFilter, FrameworkFilter, MetricFilter,
//!     LanguageFilter, Framework, PackageManager, ComparisonOp, ProgrammingLanguage,
//! };
//!
//! let filter = AdvancedRepoFilter::new()
//!     .with_dependency(
//!         DependencyFilter::new()
//!             .package_manager(PackageManager::Npm)
//!             .has("react")
//!             .has_at_least("typescript", "4.0")
//!     )
//!     .with_framework(
//!         FrameworkFilter::new()
//!             .uses(Framework::NextJs)
//!     )
//!     .with_metrics(
//!         MetricFilter::new()
//!             .lines_of_code(ComparisonOp::GreaterThan, 1000.0)
//!             .file_count(ComparisonOp::LessThan, 500.0)
//!     )
//!     .with_language(
//!         LanguageFilter::new()
//!             .primary(ProgrammingLanguage::TypeScript)
//!     );
//! ```

pub mod dependency;
pub mod framework;
pub mod language;
pub mod metrics;

pub use dependency::{DependencyFilter, DependencyInfo, PackageManager, VersionConstraint};
pub use framework::{Framework, FrameworkCategory, FrameworkFilter, FrameworkInfo};
pub use language::{LanguageFilter, LanguageInfo, ProgrammingLanguage};
pub use metrics::{ComparisonOp, MetricCondition, MetricFilter, RepositoryMetrics};

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Advanced repository filter combining multiple filter types.
#[derive(Debug, Clone, Default)]
pub struct AdvancedRepoFilter {
    /// Dependency filters.
    dependency_filters: Vec<DependencyFilter>,
    /// Framework filters.
    framework_filters: Vec<FrameworkFilter>,
    /// Metric filters.
    metric_filters: Vec<MetricFilter>,
    /// Language filters.
    language_filters: Vec<LanguageFilter>,
    /// Match mode for multiple filters.
    match_mode: MatchMode,
}

/// How to combine multiple filters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MatchMode {
    /// All filters must match (AND).
    #[default]
    All,
    /// Any filter must match (OR).
    Any,
}

impl AdvancedRepoFilter {
    /// Create a new advanced filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a dependency filter.
    pub fn with_dependency(mut self, filter: DependencyFilter) -> Self {
        self.dependency_filters.push(filter);
        self
    }

    /// Add a framework filter.
    pub fn with_framework(mut self, filter: FrameworkFilter) -> Self {
        self.framework_filters.push(filter);
        self
    }

    /// Add a metric filter.
    pub fn with_metrics(mut self, filter: MetricFilter) -> Self {
        self.metric_filters.push(filter);
        self
    }

    /// Add a language filter.
    pub fn with_language(mut self, filter: LanguageFilter) -> Self {
        self.language_filters.push(filter);
        self
    }

    /// Set match mode to OR (any filter matches).
    pub fn match_any(mut self) -> Self {
        self.match_mode = MatchMode::Any;
        self
    }

    /// Set match mode to AND (all filters must match).
    pub fn match_all(mut self) -> Self {
        self.match_mode = MatchMode::All;
        self
    }

    /// Check if a repository matches all filters.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        match self.match_mode {
            MatchMode::All => self.matches_all(repo_path),
            MatchMode::Any => self.matches_any(repo_path),
        }
    }

    /// Check if all filters match.
    fn matches_all(&self, repo_path: &Path) -> Result<bool> {
        // Check dependency filters
        for filter in &self.dependency_filters {
            if !filter.matches(repo_path)? {
                return Ok(false);
            }
        }

        // Check framework filters
        for filter in &self.framework_filters {
            if !filter.matches(repo_path)? {
                return Ok(false);
            }
        }

        // Check metric filters
        for filter in &self.metric_filters {
            if !filter.matches(repo_path)? {
                return Ok(false);
            }
        }

        // Check language filters
        for filter in &self.language_filters {
            if !filter.matches(repo_path)? {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if any filter matches.
    fn matches_any(&self, repo_path: &Path) -> Result<bool> {
        // If no filters, match everything
        if self.dependency_filters.is_empty()
            && self.framework_filters.is_empty()
            && self.metric_filters.is_empty()
            && self.language_filters.is_empty()
        {
            return Ok(true);
        }

        // Check dependency filters
        for filter in &self.dependency_filters {
            if filter.matches(repo_path)? {
                return Ok(true);
            }
        }

        // Check framework filters
        for filter in &self.framework_filters {
            if filter.matches(repo_path)? {
                return Ok(true);
            }
        }

        // Check metric filters
        for filter in &self.metric_filters {
            if filter.matches(repo_path)? {
                return Ok(true);
            }
        }

        // Check language filters
        for filter in &self.language_filters {
            if filter.matches(repo_path)? {
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Get detailed information about a repository.
    pub fn analyze(&self, repo_path: &Path) -> Result<RepositoryInfo> {
        Ok(RepositoryInfo {
            dependencies: DependencyInfo::analyze(repo_path)?,
            frameworks: FrameworkInfo::detect(repo_path)?,
            metrics: RepositoryMetrics::analyze(repo_path)?,
            languages: LanguageInfo::analyze(repo_path)?,
        })
    }
}

/// Complete information about a repository.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Dependency information.
    pub dependencies: DependencyInfo,
    /// Framework information.
    pub frameworks: FrameworkInfo,
    /// Repository metrics.
    pub metrics: RepositoryMetrics,
    /// Language information.
    pub languages: LanguageInfo,
}

impl RepositoryInfo {
    /// Analyze a repository and collect all information.
    pub fn analyze(repo_path: &Path) -> Result<Self> {
        Ok(Self {
            dependencies: DependencyInfo::analyze(repo_path)?,
            frameworks: FrameworkInfo::detect(repo_path)?,
            metrics: RepositoryMetrics::analyze(repo_path)?,
            languages: LanguageInfo::analyze(repo_path)?,
        })
    }

    /// Get a summary of the repository.
    pub fn summary(&self) -> String {
        let mut summary = Vec::new();

        // Primary language
        if let Some(lang) = self.languages.primary {
            summary.push(format!(
                "Primary language: {} ({:.1}%)",
                lang.name(),
                self.languages.primary_percentage
            ));
        }

        // Lines of code
        summary.push(format!("Lines of code: {}", self.metrics.lines_of_code));

        // File count
        summary.push(format!("Files: {}", self.metrics.file_count));

        // Frameworks
        if !self.frameworks.frameworks.is_empty() {
            let fw_names: Vec<_> = self.frameworks.frameworks.iter().map(|f| format!("{:?}", f)).collect();
            summary.push(format!("Frameworks: {}", fw_names.join(", ")));
        }

        // Package managers
        if !self.dependencies.package_managers.is_empty() {
            let pm_names: Vec<_> = self.dependencies.package_managers.iter().map(|p| format!("{:?}", p)).collect();
            summary.push(format!("Package managers: {}", pm_names.join(", ")));
        }

        // Git stats
        if let Some(commits) = self.metrics.commit_count {
            summary.push(format!("Commits: {}", commits));
        }

        if let Some(contributors) = self.metrics.contributor_count {
            summary.push(format!("Contributors: {}", contributors));
        }

        summary.join("\n")
    }
}

/// Builder for creating common filter presets.
pub struct FilterPresets;

impl FilterPresets {
    /// Filter for Rust projects.
    pub fn rust_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().package_manager(PackageManager::Cargo))
            .with_language(LanguageFilter::new().primary(ProgrammingLanguage::Rust))
    }

    /// Filter for Node.js/TypeScript projects.
    pub fn typescript_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().package_manager(PackageManager::Npm))
            .with_language(LanguageFilter::new().primary(ProgrammingLanguage::TypeScript))
    }

    /// Filter for React projects.
    pub fn react_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().has("react"))
            .with_framework(FrameworkFilter::new().uses(Framework::React))
    }

    /// Filter for Python projects.
    pub fn python_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().package_manager(PackageManager::Pip))
            .with_language(LanguageFilter::new().primary(ProgrammingLanguage::Python))
    }

    /// Filter for Django projects.
    pub fn django_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().has("django"))
            .with_framework(FrameworkFilter::new().uses(Framework::Django))
    }

    /// Filter for Go projects.
    pub fn go_project() -> AdvancedRepoFilter {
        AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().package_manager(PackageManager::GoMod))
            .with_language(LanguageFilter::new().primary(ProgrammingLanguage::Go))
    }

    /// Filter for active projects (recent commits).
    pub fn active_project(max_days: f64) -> AdvancedRepoFilter {
        AdvancedRepoFilter::new().with_metrics(
            MetricFilter::new().commit_age_days(ComparisonOp::LessThan, max_days),
        )
    }

    /// Filter for large projects.
    pub fn large_project(min_lines: f64) -> AdvancedRepoFilter {
        AdvancedRepoFilter::new().with_metrics(
            MetricFilter::new().lines_of_code(ComparisonOp::GreaterThan, min_lines),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_advanced_filter_builder() {
        let filter = AdvancedRepoFilter::new()
            .with_dependency(DependencyFilter::new().has("serde"))
            .with_framework(FrameworkFilter::new().uses(Framework::ActixWeb))
            .with_metrics(MetricFilter::new().lines_of_code(ComparisonOp::GreaterThan, 1000.0))
            .with_language(LanguageFilter::new().primary(ProgrammingLanguage::Rust));

        assert_eq!(filter.dependency_filters.len(), 1);
        assert_eq!(filter.framework_filters.len(), 1);
        assert_eq!(filter.metric_filters.len(), 1);
        assert_eq!(filter.language_filters.len(), 1);
    }

    #[test]
    fn test_match_mode() {
        let filter = AdvancedRepoFilter::new()
            .match_any()
            .with_dependency(DependencyFilter::new().has("react"))
            .with_dependency(DependencyFilter::new().has("vue"));

        assert_eq!(filter.match_mode, MatchMode::Any);
    }

    #[test]
    fn test_filter_presets() {
        let rust_filter = FilterPresets::rust_project();
        assert_eq!(rust_filter.dependency_filters.len(), 1);
        assert_eq!(rust_filter.language_filters.len(), 1);

        let react_filter = FilterPresets::react_project();
        assert_eq!(react_filter.dependency_filters.len(), 1);
        assert_eq!(react_filter.framework_filters.len(), 1);
    }
}
