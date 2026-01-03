//! Dependency-based repository filtering.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Filter repositories by their dependencies.
#[derive(Debug, Clone, Default)]
pub struct DependencyFilter {
    /// Required dependencies (name -> version constraint).
    required: HashMap<String, VersionConstraint>,
    /// Excluded dependencies.
    excluded: Vec<String>,
    /// Package manager to check.
    package_manager: Option<PackageManager>,
}

/// Package manager types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PackageManager {
    /// Rust/Cargo (Cargo.toml)
    Cargo,
    /// Node.js/npm (package.json)
    Npm,
    /// Python/pip (requirements.txt, pyproject.toml, setup.py)
    Pip,
    /// Java/Maven (pom.xml)
    Maven,
    /// Java/Gradle (build.gradle)
    Gradle,
    /// Ruby/Bundler (Gemfile)
    Bundler,
    /// Go modules (go.mod)
    GoMod,
    /// .NET/NuGet (*.csproj, packages.config)
    NuGet,
    /// PHP/Composer (composer.json)
    Composer,
}

/// Version constraint for dependency matching.
#[derive(Debug, Clone, PartialEq, Eq)]
#[derive(Default)]
pub enum VersionConstraint {
    /// Any version.
    #[default]
    Any,
    /// Exact version.
    Exact(String),
    /// Minimum version (>=).
    AtLeast(String),
    /// Maximum version (<=).
    AtMost(String),
    /// Version range.
    Range { min: String, max: String },
    /// Semantic version compatible (~).
    Compatible(String),
    /// Caret version (^).
    Caret(String),
}


impl VersionConstraint {
    /// Parse a version constraint string.
    pub fn parse(s: &str) -> Self {
        let s = s.trim();

        if s.is_empty() || s == "*" {
            return Self::Any;
        }

        if let Some(rest) = s.strip_prefix(">=") {
            return Self::AtLeast(rest.trim().to_string());
        }
        if let Some(rest) = s.strip_prefix("<=") {
            return Self::AtMost(rest.trim().to_string());
        }
        if let Some(rest) = s.strip_prefix('~') {
            return Self::Compatible(rest.trim().to_string());
        }
        if let Some(rest) = s.strip_prefix('^') {
            return Self::Caret(rest.trim().to_string());
        }
        if let Some(rest) = s.strip_prefix('=') {
            return Self::Exact(rest.trim().to_string());
        }

        // Check for range
        if let Some(idx) = s.find("..") {
            let min = s[..idx].trim().to_string();
            let max = s[idx + 2..].trim().to_string();
            return Self::Range { min, max };
        }

        Self::Exact(s.to_string())
    }

    /// Check if a version satisfies this constraint.
    pub fn matches(&self, version: &str) -> bool {
        match self {
            Self::Any => true,
            Self::Exact(v) => self.versions_equal(version, v),
            Self::AtLeast(min) => self.compare_versions(version, min) >= 0,
            Self::AtMost(max) => self.compare_versions(version, max) <= 0,
            Self::Range { min, max } => {
                self.compare_versions(version, min) >= 0
                    && self.compare_versions(version, max) <= 0
            }
            Self::Compatible(base) => {
                // ~1.2.3 means >=1.2.3 <1.3.0
                if self.compare_versions(version, base) < 0 {
                    return false;
                }
                let base_parts: Vec<&str> = base.split('.').collect();
                let ver_parts: Vec<&str> = version.split('.').collect();

                if base_parts.len() >= 2 && ver_parts.len() >= 2 {
                    base_parts[0] == ver_parts[0] && base_parts[1] == ver_parts[1]
                } else {
                    true
                }
            }
            Self::Caret(base) => {
                // ^1.2.3 means >=1.2.3 <2.0.0
                if self.compare_versions(version, base) < 0 {
                    return false;
                }
                let base_parts: Vec<&str> = base.split('.').collect();
                let ver_parts: Vec<&str> = version.split('.').collect();

                if !base_parts.is_empty() && !ver_parts.is_empty() {
                    base_parts[0] == ver_parts[0]
                } else {
                    true
                }
            }
        }
    }

    /// Compare two version strings.
    fn compare_versions(&self, a: &str, b: &str) -> i32 {
        let a_parts: Vec<u32> = a.split('.').filter_map(|p| p.parse().ok()).collect();
        let b_parts: Vec<u32> = b.split('.').filter_map(|p| p.parse().ok()).collect();

        for i in 0..a_parts.len().max(b_parts.len()) {
            let a_val = a_parts.get(i).copied().unwrap_or(0);
            let b_val = b_parts.get(i).copied().unwrap_or(0);

            if a_val > b_val {
                return 1;
            }
            if a_val < b_val {
                return -1;
            }
        }

        0
    }

    /// Check if two versions are equal.
    fn versions_equal(&self, a: &str, b: &str) -> bool {
        self.compare_versions(a, b) == 0
    }
}

impl DependencyFilter {
    /// Create a new dependency filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require a specific package manager.
    pub fn package_manager(mut self, pm: PackageManager) -> Self {
        self.package_manager = Some(pm);
        self
    }

    /// Require a dependency with any version.
    pub fn has(mut self, name: impl Into<String>) -> Self {
        self.required.insert(name.into(), VersionConstraint::Any);
        self
    }

    /// Require a dependency with a specific version.
    pub fn has_version(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.required.insert(name.into(), VersionConstraint::parse(&version.into()));
        self
    }

    /// Require a dependency with at least the specified version.
    pub fn has_at_least(mut self, name: impl Into<String>, version: impl Into<String>) -> Self {
        self.required.insert(name.into(), VersionConstraint::AtLeast(version.into()));
        self
    }

    /// Exclude a dependency.
    pub fn excludes(mut self, name: impl Into<String>) -> Self {
        self.excluded.push(name.into());
        self
    }

    /// Check if a repository matches this filter.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        // Detect package manager if not specified
        let detected = self.detect_package_managers(repo_path);

        if let Some(required_pm) = &self.package_manager
            && !detected.contains(required_pm) {
                return Ok(false);
            }

        // Check dependencies for each detected package manager
        for pm in &detected {
            let deps = self.read_dependencies(repo_path, *pm)?;

            // Check required dependencies
            for (name, constraint) in &self.required {
                if let Some(version) = deps.get(name) {
                    if !constraint.matches(version) {
                        return Ok(false);
                    }
                } else {
                    return Ok(false);
                }
            }

            // Check excluded dependencies
            for name in &self.excluded {
                if deps.contains_key(name) {
                    return Ok(false);
                }
            }
        }

        // If no package managers detected and we have requirements, fail
        if detected.is_empty() && (!self.required.is_empty() || !self.excluded.is_empty()) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Detect package managers in a repository.
    fn detect_package_managers(&self, repo_path: &Path) -> Vec<PackageManager> {
        let mut managers = Vec::new();

        if repo_path.join("Cargo.toml").exists() {
            managers.push(PackageManager::Cargo);
        }
        if repo_path.join("package.json").exists() {
            managers.push(PackageManager::Npm);
        }
        if repo_path.join("requirements.txt").exists()
            || repo_path.join("pyproject.toml").exists()
            || repo_path.join("setup.py").exists()
        {
            managers.push(PackageManager::Pip);
        }
        if repo_path.join("pom.xml").exists() {
            managers.push(PackageManager::Maven);
        }
        if repo_path.join("build.gradle").exists() || repo_path.join("build.gradle.kts").exists() {
            managers.push(PackageManager::Gradle);
        }
        if repo_path.join("Gemfile").exists() {
            managers.push(PackageManager::Bundler);
        }
        if repo_path.join("go.mod").exists() {
            managers.push(PackageManager::GoMod);
        }
        if repo_path.join("packages.config").exists()
            || self.has_csproj(repo_path)
        {
            managers.push(PackageManager::NuGet);
        }
        if repo_path.join("composer.json").exists() {
            managers.push(PackageManager::Composer);
        }

        managers
    }

    /// Check if directory has .csproj files.
    fn has_csproj(&self, repo_path: &Path) -> bool {
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension()
                    && ext == "csproj" {
                        return true;
                    }
            }
        }
        false
    }

    /// Read dependencies from a package manager manifest.
    fn read_dependencies(&self, repo_path: &Path, pm: PackageManager) -> Result<HashMap<String, String>> {
        match pm {
            PackageManager::Cargo => self.read_cargo_deps(repo_path),
            PackageManager::Npm => self.read_npm_deps(repo_path),
            PackageManager::Pip => self.read_pip_deps(repo_path),
            PackageManager::Maven => self.read_maven_deps(repo_path),
            PackageManager::Gradle => self.read_gradle_deps(repo_path),
            PackageManager::Bundler => self.read_bundler_deps(repo_path),
            PackageManager::GoMod => self.read_gomod_deps(repo_path),
            PackageManager::NuGet => self.read_nuget_deps(repo_path),
            PackageManager::Composer => self.read_composer_deps(repo_path),
        }
    }

    /// Read Cargo.toml dependencies.
    fn read_cargo_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let cargo_path = repo_path.join("Cargo.toml");

        if let Ok(content) = std::fs::read_to_string(&cargo_path) {
            // Simple TOML parsing for dependencies
            let mut in_deps = false;
            let mut in_dev_deps = false;

            for line in content.lines() {
                let line = line.trim();

                if line == "[dependencies]" {
                    in_deps = true;
                    in_dev_deps = false;
                } else if line == "[dev-dependencies]" {
                    in_deps = false;
                    in_dev_deps = true;
                } else if line.starts_with('[') {
                    in_deps = false;
                    in_dev_deps = false;
                } else if (in_deps || in_dev_deps) && line.contains('=') {
                    // Parse: name = "version" or name = { version = "..." }
                    if let Some(eq_pos) = line.find('=') {
                        let name = line[..eq_pos].trim().to_string();
                        let value = line[eq_pos + 1..].trim();

                        let version = if value.starts_with('"') {
                            // Simple version string
                            value.trim_matches('"').to_string()
                        } else if value.starts_with('{') {
                            // Table with version
                            self.extract_version_from_table(value)
                        } else {
                            "*".to_string()
                        };

                        deps.insert(name, version);
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Extract version from a TOML table string.
    fn extract_version_from_table(&self, table: &str) -> String {
        // Look for version = "..."
        if let Some(ver_pos) = table.find("version") {
            let after = &table[ver_pos + 7..];
            if let Some(eq_pos) = after.find('=') {
                let value = after[eq_pos + 1..].trim();
                if value.starts_with('"')
                    && let Some(end) = value[1..].find('"') {
                        return value[1..end + 1].to_string();
                    }
            }
        }
        "*".to_string()
    }

    /// Read package.json dependencies.
    fn read_npm_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let pkg_path = repo_path.join("package.json");

        if let Ok(content) = std::fs::read_to_string(&pkg_path) {
            // Simple JSON parsing
            self.parse_json_deps(&content, "dependencies", &mut deps);
            self.parse_json_deps(&content, "devDependencies", &mut deps);
            self.parse_json_deps(&content, "peerDependencies", &mut deps);
        }

        Ok(deps)
    }

    /// Parse dependencies from JSON content.
    fn parse_json_deps(&self, content: &str, section: &str, deps: &mut HashMap<String, String>) {
        let pattern = format!("\"{}\"", section);
        if let Some(start) = content.find(&pattern) {
            let after = &content[start + pattern.len()..];
            if let Some(brace_start) = after.find('{') {
                let after_brace = &after[brace_start + 1..];
                if let Some(brace_end) = after_brace.find('}') {
                    let deps_section = &after_brace[..brace_end];

                    // Parse "name": "version" pairs
                    for line in deps_section.split(',') {
                        let line = line.trim();
                        if let Some(colon) = line.find(':') {
                            let name = line[..colon].trim().trim_matches('"').to_string();
                            let version = line[colon + 1..].trim().trim_matches('"').to_string();
                            if !name.is_empty() {
                                deps.insert(name, version);
                            }
                        }
                    }
                }
            }
        }
    }

    /// Read requirements.txt or pyproject.toml dependencies.
    fn read_pip_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();

        // Try requirements.txt first
        let req_path = repo_path.join("requirements.txt");
        if let Ok(content) = std::fs::read_to_string(&req_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }

                // Parse: package==version, package>=version, package
                let (name, version) = if let Some(eq) = line.find("==") {
                    (line[..eq].to_string(), line[eq + 2..].to_string())
                } else if let Some(gte) = line.find(">=") {
                    (line[..gte].to_string(), line[gte..].to_string())
                } else if let Some(lte) = line.find("<=") {
                    (line[..lte].to_string(), line[lte..].to_string())
                } else {
                    (line.to_string(), "*".to_string())
                };

                deps.insert(name, version);
            }
        }

        // Also try pyproject.toml
        let pyproject_path = repo_path.join("pyproject.toml");
        if let Ok(content) = std::fs::read_to_string(&pyproject_path)
            && let Some(deps_start) = content.find("dependencies") {
                let after = &content[deps_start..];
                if let Some(bracket_start) = after.find('[') {
                    let after_bracket = &after[bracket_start + 1..];
                    if let Some(bracket_end) = after_bracket.find(']') {
                        let deps_section = &after_bracket[..bracket_end];
                        for item in deps_section.split(',') {
                            let item = item.trim().trim_matches('"').trim_matches('\'');
                            if let Some(op_pos) = item.find(['>', '<', '=', '~']) {
                                let name = item[..op_pos].to_string();
                                let version = item[op_pos..].to_string();
                                deps.insert(name, version);
                            } else if !item.is_empty() {
                                deps.insert(item.to_string(), "*".to_string());
                            }
                        }
                    }
                }
            }

        Ok(deps)
    }

    /// Read Maven pom.xml dependencies.
    fn read_maven_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let pom_path = repo_path.join("pom.xml");

        if let Ok(content) = std::fs::read_to_string(&pom_path) {
            // Simple XML parsing for dependencies
            let mut in_dependency = false;
            let mut current_group = String::new();
            let mut current_artifact = String::new();
            let mut current_version = String::new();

            for line in content.lines() {
                let line = line.trim();

                if line.contains("<dependency>") {
                    in_dependency = true;
                    current_group.clear();
                    current_artifact.clear();
                    current_version.clear();
                } else if line.contains("</dependency>") {
                    if in_dependency && !current_artifact.is_empty() {
                        let name = if current_group.is_empty() {
                            current_artifact.clone()
                        } else {
                            format!("{}:{}", current_group, current_artifact)
                        };
                        deps.insert(name, if current_version.is_empty() { "*".to_string() } else { current_version.clone() });
                    }
                    in_dependency = false;
                } else if in_dependency {
                    if let Some(group) = self.extract_xml_value(line, "groupId") {
                        current_group = group;
                    }
                    if let Some(artifact) = self.extract_xml_value(line, "artifactId") {
                        current_artifact = artifact;
                    }
                    if let Some(version) = self.extract_xml_value(line, "version") {
                        current_version = version;
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Extract value from XML element.
    fn extract_xml_value(&self, line: &str, tag: &str) -> Option<String> {
        let open_tag = format!("<{}>", tag);
        let close_tag = format!("</{}>", tag);

        if let Some(start) = line.find(&open_tag) {
            let after = &line[start + open_tag.len()..];
            if let Some(end) = after.find(&close_tag) {
                return Some(after[..end].to_string());
            }
        }
        None
    }

    /// Read Gradle build.gradle dependencies.
    fn read_gradle_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();

        for gradle_file in ["build.gradle", "build.gradle.kts"] {
            let gradle_path = repo_path.join(gradle_file);
            if let Ok(content) = std::fs::read_to_string(&gradle_path) {
                // Look for implementation/api/compile dependencies
                for line in content.lines() {
                    let line = line.trim();

                    // implementation 'group:artifact:version'
                    // implementation("group:artifact:version")
                    if (line.starts_with("implementation")
                        || line.starts_with("api")
                        || line.starts_with("compile")
                        || line.starts_with("testImplementation"))
                        && let Some(dep) = self.extract_gradle_dep(line) {
                            let parts: Vec<&str> = dep.split(':').collect();
                            if parts.len() >= 2 {
                                let name = format!("{}:{}", parts[0], parts[1]);
                                let version = parts.get(2).unwrap_or(&"*").to_string();
                                deps.insert(name, version);
                            }
                        }
                }
            }
        }

        Ok(deps)
    }

    /// Extract dependency from Gradle line.
    fn extract_gradle_dep(&self, line: &str) -> Option<String> {
        // Look for quoted string
        let start = line.find(['\'', '"'])?;
        let quote = line.chars().nth(start)?;
        let after = &line[start + 1..];
        let end = after.find(quote)?;
        Some(after[..end].to_string())
    }

    /// Read Gemfile dependencies.
    fn read_bundler_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let gemfile_path = repo_path.join("Gemfile");

        if let Ok(content) = std::fs::read_to_string(&gemfile_path) {
            for line in content.lines() {
                let line = line.trim();
                if let Some(after_gem) = line.strip_prefix("gem ") {
                    let after_gem = after_gem.trim();
                    let parts: Vec<&str> = after_gem.split(',').collect();
                    if !parts.is_empty() {
                        let name = parts[0].trim().trim_matches(|c| c == '\'' || c == '"').to_string();
                        let version = parts.get(1)
                            .map(|v| v.trim().trim_matches(|c| c == '\'' || c == '"').to_string())
                            .unwrap_or_else(|| "*".to_string());
                        deps.insert(name, version);
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Read go.mod dependencies.
    fn read_gomod_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let gomod_path = repo_path.join("go.mod");

        if let Ok(content) = std::fs::read_to_string(&gomod_path) {
            let mut in_require = false;

            for line in content.lines() {
                let line = line.trim();

                if line == "require (" {
                    in_require = true;
                } else if line == ")" {
                    in_require = false;
                } else if in_require || line.starts_with("require ") {
                    let parts: Vec<&str> = line
                        .trim_start_matches("require ")
                        .split_whitespace()
                        .collect();

                    if parts.len() >= 2 {
                        let name = parts[0].to_string();
                        let version = parts[1].to_string();
                        deps.insert(name, version);
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Read NuGet dependencies.
    fn read_nuget_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();

        // Check .csproj files
        if let Ok(entries) = std::fs::read_dir(repo_path) {
            for entry in entries.flatten() {
                if entry.path().extension().is_some_and(|e| e == "csproj")
                    && let Ok(content) = std::fs::read_to_string(entry.path()) {
                        for line in content.lines() {
                            // <PackageReference Include="Name" Version="1.0.0" />
                            if line.contains("PackageReference")
                                && let (Some(name), Some(version)) = (
                                    self.extract_xml_attr(line, "Include"),
                                    self.extract_xml_attr(line, "Version"),
                                ) {
                                    deps.insert(name, version);
                                }
                        }
                    }
            }
        }

        Ok(deps)
    }

    /// Extract XML attribute value.
    fn extract_xml_attr(&self, line: &str, attr: &str) -> Option<String> {
        let pattern = format!("{}=\"", attr);
        if let Some(start) = line.find(&pattern) {
            let after = &line[start + pattern.len()..];
            if let Some(end) = after.find('"') {
                return Some(after[..end].to_string());
            }
        }
        None
    }

    /// Read Composer dependencies.
    fn read_composer_deps(&self, repo_path: &Path) -> Result<HashMap<String, String>> {
        let mut deps = HashMap::new();
        let composer_path = repo_path.join("composer.json");

        if let Ok(content) = std::fs::read_to_string(&composer_path) {
            self.parse_json_deps(&content, "require", &mut deps);
            self.parse_json_deps(&content, "require-dev", &mut deps);
        }

        Ok(deps)
    }
}

/// Detected dependencies in a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Package managers detected.
    pub package_managers: Vec<PackageManager>,
    /// Dependencies by package manager.
    pub dependencies: HashMap<PackageManager, HashMap<String, String>>,
}

impl DependencyInfo {
    /// Analyze dependencies in a repository.
    pub fn analyze(repo_path: &Path) -> Result<Self> {
        let filter = DependencyFilter::new();
        let managers = filter.detect_package_managers(repo_path);

        let mut dependencies = HashMap::new();
        for pm in &managers {
            let deps = filter.read_dependencies(repo_path, *pm)?;
            dependencies.insert(*pm, deps);
        }

        Ok(Self {
            package_managers: managers,
            dependencies,
        })
    }

    /// Check if a dependency exists.
    pub fn has_dependency(&self, name: &str) -> bool {
        self.dependencies.values().any(|deps| deps.contains_key(name))
    }

    /// Get the version of a dependency.
    pub fn get_version(&self, name: &str) -> Option<&String> {
        for deps in self.dependencies.values() {
            if let Some(version) = deps.get(name) {
                return Some(version);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constraint_parse() {
        assert!(matches!(VersionConstraint::parse("*"), VersionConstraint::Any));
        assert!(matches!(VersionConstraint::parse(""), VersionConstraint::Any));
        assert!(matches!(VersionConstraint::parse(">=1.0"), VersionConstraint::AtLeast(_)));
        assert!(matches!(VersionConstraint::parse("<=2.0"), VersionConstraint::AtMost(_)));
        assert!(matches!(VersionConstraint::parse("~1.2"), VersionConstraint::Compatible(_)));
        assert!(matches!(VersionConstraint::parse("^1.2"), VersionConstraint::Caret(_)));
        assert!(matches!(VersionConstraint::parse("1.0.0"), VersionConstraint::Exact(_)));
    }

    #[test]
    fn test_version_constraint_matches() {
        assert!(VersionConstraint::Any.matches("1.0.0"));
        assert!(VersionConstraint::Exact("1.0.0".to_string()).matches("1.0.0"));
        assert!(!VersionConstraint::Exact("1.0.0".to_string()).matches("1.0.1"));

        assert!(VersionConstraint::AtLeast("1.0.0".to_string()).matches("1.0.0"));
        assert!(VersionConstraint::AtLeast("1.0.0".to_string()).matches("1.0.1"));
        assert!(!VersionConstraint::AtLeast("1.0.0".to_string()).matches("0.9.0"));

        assert!(VersionConstraint::Caret("1.0.0".to_string()).matches("1.5.0"));
        assert!(!VersionConstraint::Caret("1.0.0".to_string()).matches("2.0.0"));
    }

    #[test]
    fn test_compare_versions() {
        let constraint = VersionConstraint::Any;
        assert_eq!(constraint.compare_versions("1.0.0", "1.0.0"), 0);
        assert_eq!(constraint.compare_versions("1.0.1", "1.0.0"), 1);
        assert_eq!(constraint.compare_versions("1.0.0", "1.0.1"), -1);
        assert_eq!(constraint.compare_versions("2.0.0", "1.9.9"), 1);
    }

    #[test]
    fn test_dependency_filter_builder() {
        let filter = DependencyFilter::new()
            .package_manager(PackageManager::Cargo)
            .has("serde")
            .has_version("tokio", ">=1.0")
            .excludes("deprecated-crate");

        assert_eq!(filter.package_manager, Some(PackageManager::Cargo));
        assert!(filter.required.contains_key("serde"));
        assert!(filter.required.contains_key("tokio"));
        assert!(filter.excluded.contains(&"deprecated-crate".to_string()));
    }
}
