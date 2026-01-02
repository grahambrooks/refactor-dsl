//! Repository filtering for codemod operations.

use crate::codemod::RepoInfo;
use crate::error::Result;
use regex::Regex;

/// Filter for selecting which repositories to process.
///
/// Provides a fluent API for building predicates that filter repositories
/// based on various criteria like file presence, topics, and custom logic.
#[derive(Default)]
pub struct RepoFilter {
    has_files: Vec<String>,
    has_topics: Vec<String>,
    exclude_topics: Vec<String>,
    exclude_archived: bool,
    exclude_forks: bool,
    name_patterns: Vec<String>,
    custom_predicates: Vec<Box<dyn Fn(&RepoInfo) -> bool + Send + Sync>>,
}

impl RepoFilter {
    /// Create a new empty filter (matches all repositories).
    pub fn new() -> Self {
        Self::default()
    }

    /// Require the repository to contain a specific file.
    ///
    /// # Example
    ///
    /// ```rust
    /// use refactor_dsl::codemod::RepoFilter;
    ///
    /// let filter = RepoFilter::new()
    ///     .has_file("package.json")
    ///     .has_file("angular.json");
    /// ```
    pub fn has_file(mut self, path: impl Into<String>) -> Self {
        self.has_files.push(path.into());
        self
    }

    /// Require the repository to have a specific topic/tag.
    ///
    /// Topic matching is case-insensitive.
    pub fn topic(mut self, topic: impl Into<String>) -> Self {
        self.has_topics.push(topic.into());
        self
    }

    /// Exclude repositories with a specific topic.
    pub fn exclude_topic(mut self, topic: impl Into<String>) -> Self {
        self.exclude_topics.push(topic.into());
        self
    }

    /// Exclude archived repositories.
    pub fn not_archived(mut self) -> Self {
        self.exclude_archived = true;
        self
    }

    /// Exclude forked repositories.
    pub fn not_fork(mut self) -> Self {
        self.exclude_forks = true;
        self
    }

    /// Require repository name to match a regex pattern.
    pub fn name_matches(mut self, pattern: impl Into<String>) -> Self {
        self.name_patterns.push(pattern.into());
        self
    }

    /// Add a custom predicate function.
    ///
    /// # Example
    ///
    /// ```rust
    /// use refactor_dsl::codemod::RepoFilter;
    ///
    /// let filter = RepoFilter::new()
    ///     .filter(|repo| repo.name.starts_with("frontend-"));
    /// ```
    pub fn filter<F>(mut self, f: F) -> Self
    where
        F: Fn(&RepoInfo) -> bool + Send + Sync + 'static,
    {
        self.custom_predicates.push(Box::new(f));
        self
    }

    // Convenience methods for common project types

    /// Filter for Angular projects (has angular.json).
    pub fn has_angular(self) -> Self {
        self.has_file("angular.json")
    }

    /// Filter for React projects (has package.json with react dependency).
    ///
    /// Note: This only checks for package.json existence. For accurate React
    /// detection, use a custom filter that checks package.json contents.
    pub fn has_react(self) -> Self {
        self.has_file("package.json")
    }

    /// Filter for Rust projects (has Cargo.toml).
    pub fn has_rust(self) -> Self {
        self.has_file("Cargo.toml")
    }

    /// Filter for Python projects (has pyproject.toml or setup.py).
    pub fn has_python(self) -> Self {
        // Can't easily do OR with the current API, so check pyproject.toml
        self.has_file("pyproject.toml")
    }

    /// Filter for Node.js projects (has package.json).
    pub fn has_node(self) -> Self {
        self.has_file("package.json")
    }

    /// Filter for TypeScript projects (has tsconfig.json).
    pub fn has_typescript(self) -> Self {
        self.has_file("tsconfig.json")
    }

    /// Test if a repository matches this filter.
    pub fn matches(&self, repo: &RepoInfo) -> Result<bool> {
        // Check file existence
        for file in &self.has_files {
            if !repo.local_path.join(file).exists() {
                return Ok(false);
            }
        }

        // Check required topics
        for topic in &self.has_topics {
            if !repo.topics.iter().any(|t| t.eq_ignore_ascii_case(topic)) {
                return Ok(false);
            }
        }

        // Check excluded topics
        for topic in &self.exclude_topics {
            if repo.topics.iter().any(|t| t.eq_ignore_ascii_case(topic)) {
                return Ok(false);
            }
        }

        // Check name patterns
        for pattern in &self.name_patterns {
            let re = Regex::new(pattern)?;
            if !re.is_match(&repo.name) {
                return Ok(false);
            }
        }

        // Run custom predicates
        for pred in &self.custom_predicates {
            if !pred(repo) {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Check if archived repos should be excluded.
    pub fn excludes_archived(&self) -> bool {
        self.exclude_archived
    }

    /// Check if fork repos should be excluded.
    pub fn excludes_forks(&self) -> bool {
        self.exclude_forks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn test_repo(name: &str) -> RepoInfo {
        RepoInfo {
            name: name.to_string(),
            full_name: format!("org/{}", name),
            local_path: PathBuf::from("/tmp/test"),
            remote_url: None,
            default_branch: "main".to_string(),
            topics: vec!["angular".to_string(), "frontend".to_string()],
        }
    }

    #[test]
    fn test_empty_filter_matches_all() {
        let filter = RepoFilter::new();
        let repo = test_repo("test-app");
        assert!(filter.matches(&repo).unwrap());
    }

    #[test]
    fn test_topic_filter() {
        let filter = RepoFilter::new().topic("angular");
        let repo = test_repo("test-app");
        assert!(filter.matches(&repo).unwrap());

        let filter = RepoFilter::new().topic("react");
        assert!(!filter.matches(&repo).unwrap());
    }

    #[test]
    fn test_exclude_topic() {
        let filter = RepoFilter::new().exclude_topic("deprecated");
        let repo = test_repo("test-app");
        assert!(filter.matches(&repo).unwrap());

        let filter = RepoFilter::new().exclude_topic("angular");
        assert!(!filter.matches(&repo).unwrap());
    }

    #[test]
    fn test_name_pattern() {
        let filter = RepoFilter::new().name_matches("^test-");
        let repo = test_repo("test-app");
        assert!(filter.matches(&repo).unwrap());

        let filter = RepoFilter::new().name_matches("^prod-");
        assert!(!filter.matches(&repo).unwrap());
    }

    #[test]
    fn test_custom_predicate() {
        let filter = RepoFilter::new().filter(|r| r.name.contains("app"));
        let repo = test_repo("test-app");
        assert!(filter.matches(&repo).unwrap());

        let filter = RepoFilter::new().filter(|r| r.name.contains("service"));
        assert!(!filter.matches(&repo).unwrap());
    }
}
