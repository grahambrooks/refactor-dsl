//! File matching predicates.

use crate::error::Result;
use globset::{Glob, GlobSet, GlobSetBuilder};
use regex::Regex;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Predicates for matching files in a project.
#[derive(Default, Clone)]
pub struct FileMatcher {
    extensions: Vec<String>,
    include_globs: Vec<String>,
    exclude_globs: Vec<String>,
    content_patterns: Vec<String>,
    name_patterns: Vec<String>,
    min_size: Option<u64>,
    max_size: Option<u64>,
}

impl FileMatcher {
    /// Creates a new file matcher.
    pub fn new() -> Self {
        Self::default()
    }

    /// Matches files with the given extension (without dot).
    pub fn extension(mut self, ext: impl Into<String>) -> Self {
        self.extensions.push(ext.into());
        self
    }

    /// Matches files with any of the given extensions.
    pub fn extensions(mut self, exts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extensions.extend(exts.into_iter().map(Into::into));
        self
    }

    /// Includes files matching the glob pattern.
    pub fn include(mut self, pattern: impl Into<String>) -> Self {
        self.include_globs.push(pattern.into());
        self
    }

    /// Excludes files matching the glob pattern.
    pub fn exclude(mut self, pattern: impl Into<String>) -> Self {
        self.exclude_globs.push(pattern.into());
        self
    }

    /// Matches files containing the given regex pattern.
    pub fn contains_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.content_patterns.push(pattern.into());
        self
    }

    /// Matches files whose name matches the regex pattern.
    pub fn name_matches(mut self, pattern: impl Into<String>) -> Self {
        self.name_patterns.push(pattern.into());
        self
    }

    /// Matches files larger than the given size in bytes.
    pub fn min_size(mut self, bytes: u64) -> Self {
        self.min_size = Some(bytes);
        self
    }

    /// Matches files smaller than the given size in bytes.
    pub fn max_size(mut self, bytes: u64) -> Self {
        self.max_size = Some(bytes);
        self
    }

    /// Collects all matching files from the given root directory.
    pub fn collect(&self, root: &Path) -> Result<Vec<PathBuf>> {
        let include_set = self.build_glob_set(&self.include_globs)?;
        let exclude_set = self.build_glob_set(&self.exclude_globs)?;
        let content_regexes = self.compile_patterns(&self.content_patterns)?;
        let name_regexes = self.compile_patterns(&self.name_patterns)?;

        let mut matched = Vec::new();

        for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check extension
            if !self.extensions.is_empty() {
                let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                if !self.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)) {
                    continue;
                }
            }

            // Get relative path for glob matching
            let rel_path = path.strip_prefix(root).unwrap_or(path);

            // Check include globs
            if !self.include_globs.is_empty() && !include_set.is_match(rel_path) {
                continue;
            }

            // Check exclude globs
            if !self.exclude_globs.is_empty() && exclude_set.is_match(rel_path) {
                continue;
            }

            // Check name patterns
            if !name_regexes.is_empty() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !name_regexes.iter().any(|re| re.is_match(name)) {
                    continue;
                }
            }

            // Check file size
            if let Ok(metadata) = fs::metadata(path) {
                let size = metadata.len();
                if let Some(min) = self.min_size
                    && size < min
                {
                    continue;
                }
                if let Some(max) = self.max_size
                    && size > max
                {
                    continue;
                }
            }

            // Check content patterns (expensive - do last)
            if !content_regexes.is_empty() {
                if let Ok(content) = fs::read_to_string(path) {
                    if !content_regexes.iter().any(|re| re.is_match(&content)) {
                        continue;
                    }
                } else {
                    continue;
                }
            }

            matched.push(path.to_path_buf());
        }

        Ok(matched)
    }

    fn build_glob_set(&self, patterns: &[String]) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pattern in patterns {
            builder.add(Glob::new(pattern)?);
        }
        Ok(builder.build()?)
    }

    fn compile_patterns(&self, patterns: &[String]) -> Result<Vec<Regex>> {
        patterns.iter().map(|p| Ok(Regex::new(p)?)).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_files(dir: &Path) {
        // Create some test files
        fs::create_dir_all(dir.join("src")).unwrap();
        fs::create_dir_all(dir.join("tests")).unwrap();

        File::create(dir.join("src/main.rs"))
            .unwrap()
            .write_all(b"fn main() { println!(\"hello\"); }")
            .unwrap();

        File::create(dir.join("src/lib.rs"))
            .unwrap()
            .write_all(b"pub fn add(a: i32, b: i32) -> i32 { a + b }")
            .unwrap();

        File::create(dir.join("src/utils.ts"))
            .unwrap()
            .write_all(b"export function util() { return 42; }")
            .unwrap();

        File::create(dir.join("tests/test_main.rs"))
            .unwrap()
            .write_all(b"#[test] fn it_works() { assert!(true); }")
            .unwrap();

        File::create(dir.join("README.md"))
            .unwrap()
            .write_all(b"# Test Project\n\nTODO: add docs")
            .unwrap();
    }

    #[test]
    fn test_filter_by_extension() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().extension("rs");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().all(|f| f.extension().unwrap() == "rs"));
    }

    #[test]
    fn test_filter_by_multiple_extensions() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().extensions(["rs", "ts"]);
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 4);
    }

    #[test]
    fn test_exclude_glob() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().extension("rs").exclude("**/tests/**");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 2);
        assert!(files.iter().all(|f| !f.to_string_lossy().contains("tests")));
    }

    #[test]
    fn test_include_glob() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().include("**/src/**");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 3);
        assert!(files.iter().all(|f| f.to_string_lossy().contains("src")));
    }

    #[test]
    fn test_content_pattern() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().contains_pattern("TODO");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("README"));
    }

    #[test]
    fn test_name_pattern() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new().name_matches(r"^main\.");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(
            files[0]
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("main")
        );
    }

    #[test]
    fn test_combined_filters() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        let matcher = FileMatcher::new()
            .extension("rs")
            .exclude("**/tests/**")
            .contains_pattern("fn main");
        let files = matcher.collect(dir.path()).unwrap();

        assert_eq!(files.len(), 1);
        assert!(files[0].to_string_lossy().contains("main.rs"));
    }

    #[test]
    fn test_empty_directory() {
        let dir = TempDir::new().unwrap();

        let matcher = FileMatcher::new();
        let files = matcher.collect(dir.path()).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_max_size() {
        let dir = TempDir::new().unwrap();
        create_test_files(dir.path());

        // All test files are small, so max_size of 10 should exclude most
        let matcher = FileMatcher::new().max_size(10);
        let files = matcher.collect(dir.path()).unwrap();

        assert!(files.is_empty());
    }

    #[test]
    fn test_builder_pattern() {
        let _matcher = FileMatcher::new()
            .extension("rs")
            .extension("ts")
            .include("**/src/**")
            .exclude("**/node_modules/**")
            .contains_pattern("function")
            .name_matches(r"\.rs$")
            .min_size(1)
            .max_size(1000000);

        // Just verify it builds without panic
        assert!(true);
    }
}
