//! LSP-based rename refactoring.

use crate::error::{RefactorError, Result};
use crate::lsp::LspRegistry;
use crate::lsp::client::LspClient;
use crate::lsp::config::LspServerConfig;
use crate::lsp::installer::LspInstaller;
use crate::lsp::types::{Position, WorkspaceEdit};
use std::path::PathBuf;

/// Builder for LSP-based rename operations.
pub struct LspRename {
    root_path: PathBuf,
    file_path: PathBuf,
    position: Position,
    new_name: String,
    server_config: Option<LspServerConfig>,
    dry_run: bool,
    auto_install: bool,
}

impl LspRename {
    /// Creates a new rename operation.
    ///
    /// # Arguments
    /// * `file_path` - Path to the file containing the symbol
    /// * `line` - 0-indexed line number
    /// * `column` - 0-indexed column number
    /// * `new_name` - The new name for the symbol
    pub fn new(
        file_path: impl Into<PathBuf>,
        line: u32,
        column: u32,
        new_name: impl Into<String>,
    ) -> Self {
        let file_path = file_path.into();
        let root_path = file_path
            .parent()
            .map(|p| p.to_path_buf())
            .unwrap_or_else(|| PathBuf::from("."));

        Self {
            root_path,
            file_path,
            position: Position::new(line, column),
            new_name: new_name.into(),
            server_config: None,
            dry_run: false,
            auto_install: false,
        }
    }

    /// Creates a rename from a file path and symbol name to find.
    ///
    /// This will search for the first occurrence of the symbol in the file.
    pub fn find_symbol(
        file_path: impl Into<PathBuf>,
        symbol_name: &str,
        new_name: impl Into<String>,
    ) -> Result<Self> {
        let file_path = file_path.into();
        let content = std::fs::read_to_string(&file_path)?;

        // Find the symbol in the file
        let position = find_symbol_position(&content, symbol_name).ok_or_else(|| {
            RefactorError::TransformFailed {
                message: format!(
                    "Symbol '{}' not found in {}",
                    symbol_name,
                    file_path.display()
                ),
            }
        })?;

        Ok(Self::new(
            file_path,
            position.line,
            position.character,
            new_name,
        ))
    }

    /// Sets the project root path.
    pub fn root(mut self, path: impl Into<PathBuf>) -> Self {
        self.root_path = path.into();
        self
    }

    /// Uses a custom LSP server configuration.
    pub fn server(mut self, config: LspServerConfig) -> Self {
        self.server_config = Some(config);
        self
    }

    /// Enables dry-run mode (preview without applying).
    pub fn dry_run(mut self) -> Self {
        self.dry_run = true;
        self
    }

    /// Enables auto-installation of LSP servers from the Mason registry.
    ///
    /// When enabled, if the LSP server for the file type is not found in PATH,
    /// it will be automatically downloaded and installed.
    pub fn auto_install(mut self) -> Self {
        self.auto_install = true;
        self
    }

    /// Executes the rename operation.
    pub fn execute(self) -> Result<RenameResult> {
        // Find or use the configured LSP server
        let mut config = match self.server_config.clone() {
            Some(config) => config,
            None => {
                let registry = LspRegistry::new();
                registry
                    .find_for_file(&self.file_path)
                    .cloned()
                    .ok_or_else(|| {
                        RefactorError::UnsupportedLanguage(
                            self.file_path
                                .extension()
                                .and_then(|e| e.to_str())
                                .unwrap_or("unknown")
                                .to_string(),
                        )
                    })?
            }
        };

        // Check if server exists, auto-install if enabled
        if self.auto_install && !Self::server_exists(&config.command) {
            config = Self::try_install_server(&config)?;
        }

        // Find project root
        let root_path = config
            .find_root(&self.file_path)
            .unwrap_or_else(|| self.root_path.clone());

        // Start LSP client
        let mut client = LspClient::start(&config, &root_path)?;
        client.initialize()?;

        // Open the document
        client.open_document(&self.file_path)?;

        // Small delay to let server process the document
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Perform rename
        let workspace_edit = client.rename(&self.file_path, self.position, &self.new_name)?;

        // Apply if not dry run
        if !self.dry_run && !workspace_edit.is_empty() {
            workspace_edit.apply()?;
        }

        Ok(RenameResult {
            workspace_edit,
            dry_run: self.dry_run,
        })
    }

    /// Checks if a server command exists in PATH or as an absolute path.
    fn server_exists(command: &str) -> bool {
        use std::process::Command;

        // Check if it's an absolute path
        if std::path::Path::new(command).is_absolute() {
            return std::path::Path::new(command).exists();
        }

        // Try to run with --version or --help to check if it exists
        Command::new(command).arg("--version").output().is_ok()
    }

    /// Attempts to install the LSP server from the Mason registry.
    fn try_install_server(config: &LspServerConfig) -> Result<LspServerConfig> {
        let installer = LspInstaller::new()?;

        // Map config name to Mason package name
        let package_name = Self::map_server_to_package(&config.name);

        eprintln!("Installing {} from Mason registry...", package_name);
        let binary_path = installer.install(package_name)?;
        eprintln!("Installed {} at {}", package_name, binary_path.display());

        // Create new config with installed binary path
        Ok(LspServerConfig {
            name: config.name.clone(),
            command: binary_path.to_string_lossy().to_string(),
            args: config.args.clone(),
            extensions: config.extensions.clone(),
            root_markers: config.root_markers.clone(),
        })
    }

    /// Maps our server config names to Mason package names.
    fn map_server_to_package(server_name: &str) -> &str {
        match server_name {
            "rust-analyzer" => "rust-analyzer",
            "typescript-language-server" => "typescript-language-server",
            "pyright" => "pyright",
            "gopls" => "gopls",
            "clangd" => "clangd",
            _ => server_name,
        }
    }
}

/// Result of a rename operation.
#[derive(Debug)]
pub struct RenameResult {
    /// The workspace edit containing all changes.
    pub workspace_edit: WorkspaceEdit,
    /// Whether this was a dry run.
    pub dry_run: bool,
}

impl RenameResult {
    /// Returns the number of files affected.
    pub fn file_count(&self) -> usize {
        self.workspace_edit.file_count()
    }

    /// Returns the total number of edits.
    pub fn edit_count(&self) -> usize {
        self.workspace_edit.edit_count()
    }

    /// Returns true if no changes were made.
    pub fn is_empty(&self) -> bool {
        self.workspace_edit.is_empty()
    }

    /// Generates a diff of all changes.
    pub fn diff(&self) -> Result<String> {
        let previews = self.workspace_edit.preview()?;
        let mut output = String::new();

        for (path, new_content) in &previews {
            let original = std::fs::read_to_string(path)?;
            let diff = crate::diff::unified_diff(&original, new_content, path);
            output.push_str(&diff);
            output.push('\n');
        }

        Ok(output)
    }
}

/// Finds the position of a symbol in the content.
fn find_symbol_position(content: &str, symbol: &str) -> Option<Position> {
    for (line_idx, line) in content.lines().enumerate() {
        let mut search_start = 0;
        while let Some(relative_idx) = line[search_start..].find(symbol) {
            let col_idx = search_start + relative_idx;

            // Verify it's a whole word match (not part of a larger identifier)
            let before_ok = col_idx == 0
                || !line.as_bytes()[col_idx - 1].is_ascii_alphanumeric()
                    && line.as_bytes()[col_idx - 1] != b'_';

            let after_idx = col_idx + symbol.len();
            let after_ok = after_idx >= line.len()
                || !line.as_bytes()[after_idx].is_ascii_alphanumeric()
                    && line.as_bytes()[after_idx] != b'_';

            if before_ok && after_ok {
                return Some(Position::new(line_idx as u32, col_idx as u32));
            }

            // Move past this occurrence to search for more
            search_start = col_idx + 1;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_symbol_position() {
        let content = "fn hello() {\n    let world = 42;\n}";

        let pos = find_symbol_position(content, "hello");
        assert_eq!(pos, Some(Position::new(0, 3)));

        let pos = find_symbol_position(content, "world");
        assert_eq!(pos, Some(Position::new(1, 8)));

        let pos = find_symbol_position(content, "notfound");
        assert_eq!(pos, None);
    }

    #[test]
    fn test_find_symbol_whole_word() {
        let content = "let hello_world = hello;";

        // Should find standalone "hello", not "hello" in "hello_world"
        let pos = find_symbol_position(content, "hello");
        assert_eq!(pos, Some(Position::new(0, 18)));
    }

    #[test]
    fn test_lsp_rename_builder() {
        let rename = LspRename::new("test.rs", 0, 3, "new_name")
            .root("/project")
            .dry_run();

        assert_eq!(rename.position.line, 0);
        assert_eq!(rename.position.character, 3);
        assert_eq!(rename.new_name, "new_name");
        assert!(rename.dry_run);
    }
}
