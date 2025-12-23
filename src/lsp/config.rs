//! LSP server configuration.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the LSP subsystem.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LspConfig {
    /// Timeout for LSP operations in milliseconds.
    pub timeout_ms: u64,
    /// Whether to show LSP server logs.
    pub verbose: bool,
    /// Custom server configurations.
    pub servers: Vec<LspServerConfig>,
}

impl LspConfig {
    /// Creates a new LSP configuration with default settings.
    pub fn new() -> Self {
        Self {
            timeout_ms: 30000,
            verbose: false,
            servers: Vec::new(),
        }
    }

    /// Sets the operation timeout.
    pub fn timeout(mut self, ms: u64) -> Self {
        self.timeout_ms = ms;
        self
    }

    /// Enables verbose logging.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Adds a custom server configuration.
    pub fn server(mut self, config: LspServerConfig) -> Self {
        self.servers.push(config);
        self
    }
}

/// Configuration for a specific LSP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LspServerConfig {
    /// Name of the language server.
    pub name: String,
    /// Command to start the server.
    pub command: String,
    /// Arguments to pass to the server.
    pub args: Vec<String>,
    /// File extensions this server handles.
    pub extensions: Vec<String>,
    /// Files that indicate the project root.
    pub root_markers: Vec<String>,
}

impl LspServerConfig {
    /// Creates a new server configuration.
    pub fn new(name: impl Into<String>, command: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            command: command.into(),
            args: Vec::new(),
            extensions: Vec::new(),
            root_markers: Vec::new(),
        }
    }

    /// Adds command-line arguments.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args = args.into_iter().map(Into::into).collect();
        self
    }

    /// Adds an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Sets file extensions this server handles.
    pub fn extensions(mut self, exts: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.extensions = exts.into_iter().map(Into::into).collect();
        self
    }

    /// Sets root marker files.
    pub fn root_markers(mut self, markers: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.root_markers = markers.into_iter().map(Into::into).collect();
        self
    }

    /// Checks if this server handles the given file extension.
    pub fn handles_extension(&self, ext: &str) -> bool {
        self.extensions.iter().any(|e| e.eq_ignore_ascii_case(ext))
    }

    /// Finds the project root for a given file path.
    pub fn find_root(&self, file_path: &Path) -> Option<std::path::PathBuf> {
        let mut current = file_path.parent()?;

        loop {
            for marker in &self.root_markers {
                if current.join(marker).exists() {
                    return Some(current.to_path_buf());
                }
            }

            match current.parent() {
                Some(parent) => current = parent,
                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lsp_config_builder() {
        let config = LspConfig::new().timeout(5000).verbose();

        assert_eq!(config.timeout_ms, 5000);
        assert!(config.verbose);
    }

    #[test]
    fn test_lsp_config_add_server() {
        let server = LspServerConfig::new("test", "test-cmd");
        let config = LspConfig::new().server(server);

        assert_eq!(config.servers.len(), 1);
        assert_eq!(config.servers[0].name, "test");
    }

    #[test]
    fn test_server_config_builder() {
        let config = LspServerConfig::new("rust-analyzer", "rust-analyzer")
            .args(["--verbose"])
            .extensions(["rs"])
            .root_markers(["Cargo.toml"]);

        assert_eq!(config.name, "rust-analyzer");
        assert_eq!(config.command, "rust-analyzer");
        assert_eq!(config.args, vec!["--verbose"]);
        assert_eq!(config.extensions, vec!["rs"]);
        assert_eq!(config.root_markers, vec!["Cargo.toml"]);
    }

    #[test]
    fn test_server_config_arg_chaining() {
        let config = LspServerConfig::new("test", "test")
            .arg("--stdio")
            .arg("--verbose");

        assert_eq!(config.args, vec!["--stdio", "--verbose"]);
    }

    #[test]
    fn test_handles_extension() {
        let config = LspServerConfig::new("test", "test").extensions(["rs", "RS"]);

        assert!(config.handles_extension("rs"));
        assert!(config.handles_extension("RS"));
        assert!(config.handles_extension("Rs"));
        assert!(!config.handles_extension("py"));
    }

    #[test]
    fn test_handles_extension_case_insensitive() {
        let config = LspServerConfig::new("test", "test").extensions(["ts", "tsx"]);

        assert!(config.handles_extension("TS"));
        assert!(config.handles_extension("TSX"));
        assert!(config.handles_extension("ts"));
        assert!(config.handles_extension("tsx"));
    }
}
