//! Language Server Protocol support for language-agnostic refactoring.
//!
//! This module provides LSP client functionality to enable refactoring
//! operations (rename, find references, etc.) for any language with an
//! LSP server implementation.
//!
//! ## Auto-installing LSP Servers
//!
//! The installer module can automatically download LSP servers from the Mason registry:
//!
//! ```rust,no_run
//! use refactor::lsp::installer::LspInstaller;
//!
//! let installer = LspInstaller::new()?;
//!
//! // Install rust-analyzer if not already installed
//! let binary_path = installer.ensure_installed("rust-analyzer")?;
//! println!("rust-analyzer installed at: {}", binary_path.display());
//! # Ok::<(), refactor::error::RefactorError>(())
//! ```

mod client;
mod config;
pub mod installer;
mod rename;
mod types;

pub use client::LspClient;
pub use config::{LspConfig, LspServerConfig};
pub use installer::LspInstaller;
pub use rename::{LspRename, RenameResult};
pub use types::{Location, Position, Range, TextEdit, WorkspaceEdit};

use std::path::Path;

/// Registry of LSP server configurations for different languages.
#[derive(Default)]
pub struct LspRegistry {
    servers: Vec<LspServerConfig>,
}

impl LspRegistry {
    /// Creates a new registry with common LSP server configurations.
    pub fn new() -> Self {
        let mut registry = Self::default();
        registry.register_defaults();
        registry
    }

    /// Creates an empty registry.
    pub fn empty() -> Self {
        Self::default()
    }

    /// Registers default LSP server configurations.
    fn register_defaults(&mut self) {
        // Rust - rust-analyzer
        self.register(LspServerConfig {
            name: "rust-analyzer".to_string(),
            command: "rust-analyzer".to_string(),
            args: vec![],
            extensions: vec!["rs".to_string()],
            root_markers: vec!["Cargo.toml".to_string(), "rust-project.json".to_string()],
        });

        // TypeScript/JavaScript - typescript-language-server
        self.register(LspServerConfig {
            name: "typescript-language-server".to_string(),
            command: "typescript-language-server".to_string(),
            args: vec!["--stdio".to_string()],
            extensions: vec![
                "ts".to_string(),
                "tsx".to_string(),
                "js".to_string(),
                "jsx".to_string(),
            ],
            root_markers: vec![
                "tsconfig.json".to_string(),
                "jsconfig.json".to_string(),
                "package.json".to_string(),
            ],
        });

        // Python - pyright or pylsp
        self.register(LspServerConfig {
            name: "pyright".to_string(),
            command: "pyright-langserver".to_string(),
            args: vec!["--stdio".to_string()],
            extensions: vec!["py".to_string(), "pyi".to_string()],
            root_markers: vec![
                "pyproject.toml".to_string(),
                "setup.py".to_string(),
                "pyrightconfig.json".to_string(),
            ],
        });

        // Go - gopls
        self.register(LspServerConfig {
            name: "gopls".to_string(),
            command: "gopls".to_string(),
            args: vec!["serve".to_string()],
            extensions: vec!["go".to_string()],
            root_markers: vec!["go.mod".to_string(), "go.work".to_string()],
        });

        // C/C++ - clangd
        self.register(LspServerConfig {
            name: "clangd".to_string(),
            command: "clangd".to_string(),
            args: vec![],
            extensions: vec![
                "c".to_string(),
                "h".to_string(),
                "cpp".to_string(),
                "hpp".to_string(),
                "cc".to_string(),
                "cxx".to_string(),
            ],
            root_markers: vec![
                "compile_commands.json".to_string(),
                "CMakeLists.txt".to_string(),
                ".clangd".to_string(),
            ],
        });

        // Java - jdtls (Eclipse JDT Language Server)
        self.register(LspServerConfig {
            name: "jdtls".to_string(),
            command: "jdtls".to_string(),
            args: vec![],
            extensions: vec!["java".to_string()],
            root_markers: vec![
                "pom.xml".to_string(),
                "build.gradle".to_string(),
                "build.gradle.kts".to_string(),
                ".project".to_string(),
            ],
        });

        // C# - omnisharp
        self.register(LspServerConfig {
            name: "omnisharp".to_string(),
            command: "omnisharp".to_string(),
            args: vec!["--languageserver".to_string()],
            extensions: vec!["cs".to_string(), "csx".to_string()],
            root_markers: vec![
                "*.sln".to_string(),
                "*.csproj".to_string(),
                "omnisharp.json".to_string(),
            ],
        });

        // Ruby - solargraph
        self.register(LspServerConfig {
            name: "solargraph".to_string(),
            command: "solargraph".to_string(),
            args: vec!["stdio".to_string()],
            extensions: vec!["rb".to_string(), "rake".to_string(), "gemspec".to_string()],
            root_markers: vec!["Gemfile".to_string(), ".solargraph.yml".to_string()],
        });
    }

    /// Registers a custom LSP server configuration.
    pub fn register(&mut self, config: LspServerConfig) {
        self.servers.push(config);
    }

    /// Finds an LSP server config for a given file extension.
    pub fn find_by_extension(&self, ext: &str) -> Option<&LspServerConfig> {
        self.servers
            .iter()
            .find(|s| s.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext)))
    }

    /// Finds an LSP server config for a given file path.
    pub fn find_for_file(&self, path: &Path) -> Option<&LspServerConfig> {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.find_by_extension(ext))
    }

    /// Returns all registered server configurations.
    pub fn all(&self) -> &[LspServerConfig] {
        &self.servers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_new_has_defaults() {
        let registry = LspRegistry::new();
        assert!(!registry.all().is_empty());
    }

    #[test]
    fn test_registry_empty() {
        let registry = LspRegistry::empty();
        assert!(registry.all().is_empty());
    }

    #[test]
    fn test_registry_find_rust() {
        let registry = LspRegistry::new();
        let config = registry.find_by_extension("rs");

        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "rust-analyzer");
    }

    #[test]
    fn test_registry_find_typescript() {
        let registry = LspRegistry::new();

        assert!(registry.find_by_extension("ts").is_some());
        assert!(registry.find_by_extension("tsx").is_some());
        assert!(registry.find_by_extension("js").is_some());
        assert!(registry.find_by_extension("jsx").is_some());
    }

    #[test]
    fn test_registry_find_python() {
        let registry = LspRegistry::new();
        let config = registry.find_by_extension("py");

        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "pyright");
    }

    #[test]
    fn test_registry_find_go() {
        let registry = LspRegistry::new();
        let config = registry.find_by_extension("go");

        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "gopls");
    }

    #[test]
    fn test_registry_find_cpp() {
        let registry = LspRegistry::new();

        assert!(registry.find_by_extension("c").is_some());
        assert!(registry.find_by_extension("cpp").is_some());
        assert!(registry.find_by_extension("h").is_some());
        assert!(registry.find_by_extension("hpp").is_some());
    }

    #[test]
    fn test_registry_find_unknown() {
        let registry = LspRegistry::new();
        assert!(registry.find_by_extension("xyz").is_none());
    }

    #[test]
    fn test_registry_find_for_file() {
        let registry = LspRegistry::new();

        let rust_config = registry.find_for_file(Path::new("src/main.rs"));
        assert!(rust_config.is_some());
        assert_eq!(rust_config.unwrap().name, "rust-analyzer");

        let ts_config = registry.find_for_file(Path::new("src/app.tsx"));
        assert!(ts_config.is_some());
    }

    #[test]
    fn test_registry_find_for_file_no_extension() {
        let registry = LspRegistry::new();
        let config = registry.find_for_file(Path::new("Makefile"));
        assert!(config.is_none());
    }

    #[test]
    fn test_registry_register_custom() {
        let mut registry = LspRegistry::empty();
        registry.register(LspServerConfig::new("custom", "custom-lsp").extensions(["xyz"]));

        let config = registry.find_by_extension("xyz");
        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "custom");
    }

    #[test]
    fn test_registry_case_insensitive() {
        let registry = LspRegistry::new();

        assert!(registry.find_by_extension("RS").is_some());
        assert!(registry.find_by_extension("Rs").is_some());
        assert!(registry.find_by_extension("PY").is_some());
    }

    #[test]
    fn test_registry_find_java() {
        let registry = LspRegistry::new();
        let config = registry.find_by_extension("java");

        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "jdtls");
    }

    #[test]
    fn test_registry_find_csharp() {
        let registry = LspRegistry::new();

        let config = registry.find_by_extension("cs");
        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "omnisharp");

        assert!(registry.find_by_extension("csx").is_some());
    }

    #[test]
    fn test_registry_find_ruby() {
        let registry = LspRegistry::new();

        let config = registry.find_by_extension("rb");
        assert!(config.is_some());
        assert_eq!(config.unwrap().name, "solargraph");

        assert!(registry.find_by_extension("rake").is_some());
        assert!(registry.find_by_extension("gemspec").is_some());
    }
}
