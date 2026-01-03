//! Language detection and filtering.

use std::collections::HashMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Programming languages for detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProgrammingLanguage {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Java,
    Kotlin,
    Scala,
    CSharp,
    FSharp,
    Ruby,
    PHP,
    C,
    Cpp,
    Swift,
    ObjectiveC,
    Dart,
    Elixir,
    Erlang,
    Haskell,
    Clojure,
    Lua,
    R,
    Julia,
    Shell,
    PowerShell,
    Perl,
    Vim,
    HTML,
    CSS,
    SCSS,
    SQL,
    Markdown,
    YAML,
    JSON,
    TOML,
    XML,
    Other,
}

impl ProgrammingLanguage {
    /// Get the language name.
    pub fn name(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Go => "Go",
            Self::Java => "Java",
            Self::Kotlin => "Kotlin",
            Self::Scala => "Scala",
            Self::CSharp => "C#",
            Self::FSharp => "F#",
            Self::Ruby => "Ruby",
            Self::PHP => "PHP",
            Self::C => "C",
            Self::Cpp => "C++",
            Self::Swift => "Swift",
            Self::ObjectiveC => "Objective-C",
            Self::Dart => "Dart",
            Self::Elixir => "Elixir",
            Self::Erlang => "Erlang",
            Self::Haskell => "Haskell",
            Self::Clojure => "Clojure",
            Self::Lua => "Lua",
            Self::R => "R",
            Self::Julia => "Julia",
            Self::Shell => "Shell",
            Self::PowerShell => "PowerShell",
            Self::Perl => "Perl",
            Self::Vim => "Vim script",
            Self::HTML => "HTML",
            Self::CSS => "CSS",
            Self::SCSS => "SCSS",
            Self::SQL => "SQL",
            Self::Markdown => "Markdown",
            Self::YAML => "YAML",
            Self::JSON => "JSON",
            Self::TOML => "TOML",
            Self::XML => "XML",
            Self::Other => "Other",
        }
    }

    /// Get file extensions for this language.
    pub fn extensions(&self) -> Vec<&'static str> {
        match self {
            Self::Rust => vec!["rs"],
            Self::Python => vec!["py", "pyi", "pyw"],
            Self::JavaScript => vec!["js", "mjs", "cjs", "jsx"],
            Self::TypeScript => vec!["ts", "tsx", "mts", "cts"],
            Self::Go => vec!["go"],
            Self::Java => vec!["java"],
            Self::Kotlin => vec!["kt", "kts"],
            Self::Scala => vec!["scala", "sc"],
            Self::CSharp => vec!["cs"],
            Self::FSharp => vec!["fs", "fsi", "fsx"],
            Self::Ruby => vec!["rb", "rake", "gemspec"],
            Self::PHP => vec!["php", "phtml"],
            Self::C => vec!["c", "h"],
            Self::Cpp => vec!["cpp", "cc", "cxx", "hpp", "hxx", "hh"],
            Self::Swift => vec!["swift"],
            Self::ObjectiveC => vec!["m", "mm"],
            Self::Dart => vec!["dart"],
            Self::Elixir => vec!["ex", "exs"],
            Self::Erlang => vec!["erl", "hrl"],
            Self::Haskell => vec!["hs", "lhs"],
            Self::Clojure => vec!["clj", "cljs", "cljc", "edn"],
            Self::Lua => vec!["lua"],
            Self::R => vec!["r", "R"],
            Self::Julia => vec!["jl"],
            Self::Shell => vec!["sh", "bash", "zsh", "fish"],
            Self::PowerShell => vec!["ps1", "psm1", "psd1"],
            Self::Perl => vec!["pl", "pm"],
            Self::Vim => vec!["vim"],
            Self::HTML => vec!["html", "htm"],
            Self::CSS => vec!["css"],
            Self::SCSS => vec!["scss", "sass", "less"],
            Self::SQL => vec!["sql"],
            Self::Markdown => vec!["md", "markdown"],
            Self::YAML => vec!["yml", "yaml"],
            Self::JSON => vec!["json"],
            Self::TOML => vec!["toml"],
            Self::XML => vec!["xml", "xsl", "xslt"],
            Self::Other => vec![],
        }
    }

    /// Check if this is a compiled language.
    pub fn is_compiled(&self) -> bool {
        matches!(
            self,
            Self::Rust
                | Self::Go
                | Self::Java
                | Self::Kotlin
                | Self::Scala
                | Self::CSharp
                | Self::FSharp
                | Self::C
                | Self::Cpp
                | Self::Swift
                | Self::ObjectiveC
                | Self::Dart
                | Self::Haskell
                | Self::Erlang
        )
    }

    /// Check if this is an interpreted language.
    pub fn is_interpreted(&self) -> bool {
        matches!(
            self,
            Self::Python
                | Self::JavaScript
                | Self::TypeScript
                | Self::Ruby
                | Self::PHP
                | Self::Lua
                | Self::R
                | Self::Julia
                | Self::Shell
                | Self::PowerShell
                | Self::Perl
        )
    }

    /// Check if this is a markup/data language.
    pub fn is_markup(&self) -> bool {
        matches!(
            self,
            Self::HTML | Self::CSS | Self::SCSS | Self::Markdown | Self::YAML | Self::JSON | Self::TOML | Self::XML
        )
    }

    /// Get from file extension.
    pub fn from_extension(ext: &str) -> Self {
        let ext = ext.to_lowercase();
        let ext = ext.as_str();

        for lang in Self::all() {
            if lang.extensions().contains(&ext) {
                return lang;
            }
        }

        Self::Other
    }

    /// Get all languages.
    pub fn all() -> Vec<Self> {
        vec![
            Self::Rust,
            Self::Python,
            Self::JavaScript,
            Self::TypeScript,
            Self::Go,
            Self::Java,
            Self::Kotlin,
            Self::Scala,
            Self::CSharp,
            Self::FSharp,
            Self::Ruby,
            Self::PHP,
            Self::C,
            Self::Cpp,
            Self::Swift,
            Self::ObjectiveC,
            Self::Dart,
            Self::Elixir,
            Self::Erlang,
            Self::Haskell,
            Self::Clojure,
            Self::Lua,
            Self::R,
            Self::Julia,
            Self::Shell,
            Self::PowerShell,
            Self::Perl,
            Self::Vim,
            Self::HTML,
            Self::CSS,
            Self::SCSS,
            Self::SQL,
            Self::Markdown,
            Self::YAML,
            Self::JSON,
            Self::TOML,
            Self::XML,
        ]
    }
}

/// Filter repositories by programming language.
#[derive(Debug, Clone, Default)]
pub struct LanguageFilter {
    /// Required primary language.
    required_primary: Option<ProgrammingLanguage>,
    /// Required languages (any of these).
    required_any: Vec<ProgrammingLanguage>,
    /// Required languages (all of these).
    required_all: Vec<ProgrammingLanguage>,
    /// Excluded languages.
    excluded: Vec<ProgrammingLanguage>,
    /// Minimum percentage for primary language.
    min_primary_percentage: Option<f64>,
}

impl LanguageFilter {
    /// Create a new language filter.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require a specific primary language.
    pub fn primary(mut self, lang: ProgrammingLanguage) -> Self {
        self.required_primary = Some(lang);
        self
    }

    /// Require any of the specified languages.
    pub fn any_of(mut self, langs: Vec<ProgrammingLanguage>) -> Self {
        self.required_any = langs;
        self
    }

    /// Require all of the specified languages.
    pub fn all_of(mut self, langs: Vec<ProgrammingLanguage>) -> Self {
        self.required_all = langs;
        self
    }

    /// Exclude a language.
    pub fn excludes(mut self, lang: ProgrammingLanguage) -> Self {
        self.excluded.push(lang);
        self
    }

    /// Require minimum percentage for primary language.
    pub fn min_primary_percentage(mut self, percentage: f64) -> Self {
        self.min_primary_percentage = Some(percentage);
        self
    }

    /// Check if a repository matches this filter.
    pub fn matches(&self, repo_path: &Path) -> Result<bool> {
        let info = LanguageInfo::analyze(repo_path)?;

        // Check primary language
        if let Some(ref required) = self.required_primary
            && info.primary != Some(*required) {
                return Ok(false);
            }

        // Check primary percentage
        if let Some(min_pct) = self.min_primary_percentage
            && info.primary_percentage < min_pct {
                return Ok(false);
            }

        // Check required any
        if !self.required_any.is_empty() {
            let has_any = self.required_any.iter().any(|l| info.languages.contains_key(l));
            if !has_any {
                return Ok(false);
            }
        }

        // Check required all
        for lang in &self.required_all {
            if !info.languages.contains_key(lang) {
                return Ok(false);
            }
        }

        // Check excluded
        for lang in &self.excluded {
            if info.languages.contains_key(lang) {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

/// Language information for a repository.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LanguageInfo {
    /// Languages detected with line counts.
    pub languages: HashMap<ProgrammingLanguage, usize>,
    /// Primary language (most lines).
    pub primary: Option<ProgrammingLanguage>,
    /// Primary language percentage.
    pub primary_percentage: f64,
    /// Total lines of code.
    pub total_lines: usize,
}

impl LanguageInfo {
    /// Analyze languages in a repository.
    pub fn analyze(repo_path: &Path) -> Result<Self> {
        let mut languages: HashMap<ProgrammingLanguage, usize> = HashMap::new();

        Self::walk_directory(repo_path, &mut languages)?;

        let total_lines: usize = languages.values().sum();

        let (primary, primary_percentage) = if total_lines > 0 {
            let (lang, count) = languages
                .iter()
                .max_by_key(|(_, count)| *count)
                .map(|(l, c)| (*l, *c))
                .unwrap_or((ProgrammingLanguage::Other, 0));

            (Some(lang), count as f64 / total_lines as f64 * 100.0)
        } else {
            (None, 0.0)
        };

        Ok(Self {
            languages,
            primary,
            primary_percentage,
            total_lines,
        })
    }

    /// Walk a directory and count lines by language.
    fn walk_directory(dir: &Path, languages: &mut HashMap<ProgrammingLanguage, usize>) -> Result<()> {
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
                Self::walk_directory(&path, languages)?;
            } else if path.is_file()
                && let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let lang = ProgrammingLanguage::from_extension(ext);
                    if lang != ProgrammingLanguage::Other && !lang.is_markup()
                        && let Ok(content) = std::fs::read_to_string(&path) {
                            let line_count = content.lines().count();
                            *languages.entry(lang).or_insert(0) += line_count;
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

    /// Get language distribution as percentages.
    pub fn distribution(&self) -> HashMap<ProgrammingLanguage, f64> {
        if self.total_lines == 0 {
            return HashMap::new();
        }

        self.languages
            .iter()
            .map(|(lang, count)| (*lang, *count as f64 / self.total_lines as f64 * 100.0))
            .collect()
    }

    /// Check if a language is present.
    pub fn has(&self, lang: ProgrammingLanguage) -> bool {
        self.languages.contains_key(&lang)
    }

    /// Get percentage for a specific language.
    pub fn percentage(&self, lang: ProgrammingLanguage) -> f64 {
        if self.total_lines == 0 {
            return 0.0;
        }

        self.languages
            .get(&lang)
            .map(|count| *count as f64 / self.total_lines as f64 * 100.0)
            .unwrap_or(0.0)
    }

    /// Get top N languages.
    pub fn top(&self, n: usize) -> Vec<(ProgrammingLanguage, f64)> {
        let mut dist: Vec<_> = self.distribution().into_iter().collect();
        dist.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        dist.truncate(n);
        dist
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_from_extension() {
        assert_eq!(ProgrammingLanguage::from_extension("rs"), ProgrammingLanguage::Rust);
        assert_eq!(ProgrammingLanguage::from_extension("py"), ProgrammingLanguage::Python);
        assert_eq!(ProgrammingLanguage::from_extension("js"), ProgrammingLanguage::JavaScript);
        assert_eq!(ProgrammingLanguage::from_extension("ts"), ProgrammingLanguage::TypeScript);
        assert_eq!(ProgrammingLanguage::from_extension("go"), ProgrammingLanguage::Go);
        assert_eq!(ProgrammingLanguage::from_extension("unknown"), ProgrammingLanguage::Other);
    }

    #[test]
    fn test_language_properties() {
        assert!(ProgrammingLanguage::Rust.is_compiled());
        assert!(!ProgrammingLanguage::Rust.is_interpreted());

        assert!(ProgrammingLanguage::Python.is_interpreted());
        assert!(!ProgrammingLanguage::Python.is_compiled());

        assert!(ProgrammingLanguage::HTML.is_markup());
        assert!(!ProgrammingLanguage::Rust.is_markup());
    }

    #[test]
    fn test_language_filter_builder() {
        let filter = LanguageFilter::new()
            .primary(ProgrammingLanguage::Rust)
            .min_primary_percentage(50.0)
            .excludes(ProgrammingLanguage::PHP);

        assert_eq!(filter.required_primary, Some(ProgrammingLanguage::Rust));
        assert_eq!(filter.min_primary_percentage, Some(50.0));
        assert!(filter.excluded.contains(&ProgrammingLanguage::PHP));
    }

    #[test]
    fn test_language_info_distribution() {
        let mut info = LanguageInfo::default();
        info.languages.insert(ProgrammingLanguage::Rust, 1000);
        info.languages.insert(ProgrammingLanguage::Python, 500);
        info.total_lines = 1500;

        let dist = info.distribution();
        assert!((dist[&ProgrammingLanguage::Rust] - 66.67).abs() < 1.0);
        assert!((dist[&ProgrammingLanguage::Python] - 33.33).abs() < 1.0);
    }

    #[test]
    fn test_language_info_top() {
        let mut info = LanguageInfo::default();
        info.languages.insert(ProgrammingLanguage::Rust, 1000);
        info.languages.insert(ProgrammingLanguage::Python, 500);
        info.languages.insert(ProgrammingLanguage::JavaScript, 200);
        info.total_lines = 1700;

        let top = info.top(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].0, ProgrammingLanguage::Rust);
        assert_eq!(top[1].0, ProgrammingLanguage::Python);
    }
}
