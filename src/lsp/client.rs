//! LSP client for communicating with language servers.

use crate::error::{RefactorError, Result};
use crate::lsp::config::LspServerConfig;
use crate::lsp::types::{Position, Range, TextEdit, WorkspaceEdit};
use lsp_types::{
    ClientCapabilities, DidOpenTextDocumentParams, InitializeParams, InitializedParams,
    RenameParams, TextDocumentIdentifier, TextDocumentItem, TextDocumentPositionParams,
    WorkspaceClientCapabilities, WorkspaceEditClientCapabilities,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicI64, Ordering};
use std::sync::{Arc, Mutex};
use url::Url;

/// A client for communicating with an LSP server.
pub struct LspClient {
    #[allow(dead_code)]
    config: LspServerConfig,
    root_path: PathBuf,
    process: Child,
    stdin: Arc<Mutex<ChildStdin>>,
    stdout: Arc<Mutex<BufReader<ChildStdout>>>,
    request_id: AtomicI64,
    initialized: bool,
}

impl LspClient {
    /// Creates a new LSP client and starts the server.
    pub fn start(config: &LspServerConfig, root_path: impl Into<PathBuf>) -> Result<Self> {
        let root_path = root_path.into();

        let mut process = Command::new(&config.command)
            .args(&config.args)
            .current_dir(&root_path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to start LSP server '{}': {}", config.name, e),
            })?;

        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| RefactorError::TransformFailed {
                message: "Failed to get LSP server stdin".to_string(),
            })?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| RefactorError::TransformFailed {
                message: "Failed to get LSP server stdout".to_string(),
            })?;

        Ok(Self {
            config: config.clone(),
            root_path,
            process,
            stdin: Arc::new(Mutex::new(stdin)),
            stdout: Arc::new(Mutex::new(BufReader::new(stdout))),
            request_id: AtomicI64::new(1),
            initialized: false,
        })
    }

    /// Initializes the LSP server.
    pub fn initialize(&mut self) -> Result<()> {
        if self.initialized {
            return Ok(());
        }

        let root_uri =
            Url::from_file_path(&self.root_path).map_err(|_| RefactorError::TransformFailed {
                message: "Invalid root path".to_string(),
            })?;

        let params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: Some(root_uri.clone()),
            // root_path: Some(self.root_path.to_string_lossy().to_string()),
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: Some(true),
                        ..Default::default()
                    }),
                    ..Default::default()
                }),
                ..Default::default()
            },
            workspace_folders: Some(vec![lsp_types::WorkspaceFolder {
                uri: root_uri,
                name: self
                    .root_path
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_default(),
            }]),
            ..Default::default()
        };

        let _response: Value = self.send_request("initialize", params)?;

        // Send initialized notification
        self.send_notification("initialized", InitializedParams {})?;

        self.initialized = true;
        Ok(())
    }

    /// Opens a document in the LSP server.
    pub fn open_document(&self, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let uri = Url::from_file_path(path).map_err(|_| RefactorError::TransformFailed {
            message: format!("Invalid file path: {}", path.display()),
        })?;

        let language_id = self.detect_language_id(path);

        let params = DidOpenTextDocumentParams {
            text_document: TextDocumentItem {
                uri,
                language_id,
                version: 1,
                text: content,
            },
        };

        self.send_notification("textDocument/didOpen", params)?;
        Ok(())
    }

    /// Performs a rename operation.
    pub fn rename(&self, path: &Path, position: Position, new_name: &str) -> Result<WorkspaceEdit> {
        let uri = Url::from_file_path(path).map_err(|_| RefactorError::TransformFailed {
            message: format!("Invalid file path: {}", path.display()),
        })?;

        let params = RenameParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: position.to_lsp(),
            },
            new_name: new_name.to_string(),
            work_done_progress_params: Default::default(),
        };

        let response: Option<lsp_types::WorkspaceEdit> =
            self.send_request("textDocument/rename", params)?;

        match response {
            Some(edit) => self.convert_workspace_edit(&edit),
            None => Ok(WorkspaceEdit::new()),
        }
    }

    /// Finds all references to a symbol.
    pub fn find_references(
        &self,
        path: &Path,
        position: Position,
        include_declaration: bool,
    ) -> Result<Vec<crate::lsp::types::Location>> {
        let uri = Url::from_file_path(path).map_err(|_| RefactorError::TransformFailed {
            message: format!("Invalid file path: {}", path.display()),
        })?;

        let params = lsp_types::ReferenceParams {
            text_document_position: TextDocumentPositionParams {
                text_document: TextDocumentIdentifier { uri },
                position: position.to_lsp(),
            },
            context: lsp_types::ReferenceContext {
                include_declaration,
            },
            work_done_progress_params: Default::default(),
            partial_result_params: Default::default(),
        };

        let response: Option<Vec<lsp_types::Location>> =
            self.send_request("textDocument/references", params)?;

        Ok(response
            .unwrap_or_default()
            .into_iter()
            .filter_map(|loc| self.convert_location(&loc).ok())
            .collect())
    }

    /// Goes to the definition of a symbol.
    pub fn goto_definition(
        &self,
        path: &Path,
        position: Position,
    ) -> Result<Vec<crate::lsp::types::Location>> {
        let uri = Url::from_file_path(path).map_err(|_| RefactorError::TransformFailed {
            message: format!("Invalid file path: {}", path.display()),
        })?;

        let params = TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri },
            position: position.to_lsp(),
        };

        let response: Option<lsp_types::GotoDefinitionResponse> =
            self.send_request("textDocument/definition", params)?;

        let locations = match response {
            Some(lsp_types::GotoDefinitionResponse::Scalar(loc)) => vec![loc],
            Some(lsp_types::GotoDefinitionResponse::Array(locs)) => locs,
            Some(lsp_types::GotoDefinitionResponse::Link(links)) => links
                .into_iter()
                .map(|l| lsp_types::Location {
                    uri: l.target_uri,
                    range: l.target_selection_range,
                })
                .collect(),
            None => vec![],
        };

        Ok(locations
            .into_iter()
            .filter_map(|loc| self.convert_location(&loc).ok())
            .collect())
    }

    /// Shuts down the LSP server gracefully.
    pub fn shutdown(&mut self) -> Result<()> {
        let _: Option<()> = self.send_request("shutdown", ())?;
        self.send_notification("exit", ())?;
        let _ = self.process.wait();
        Ok(())
    }

    // Private helper methods

    fn send_request<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R> {
        let id = self.request_id.fetch_add(1, Ordering::SeqCst);

        let request = json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        });

        self.send_message(&request)?;
        self.read_response(id)
    }

    fn send_notification<P: Serialize>(&self, method: &str, params: P) -> Result<()> {
        let notification = json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params
        });

        self.send_message(&notification)
    }

    fn send_message(&self, message: &Value) -> Result<()> {
        let content = serde_json::to_string(message)?;
        let header = format!("Content-Length: {}\r\n\r\n", content.len());

        let mut stdin = self
            .stdin
            .lock()
            .map_err(|_| RefactorError::TransformFailed {
                message: "Failed to lock stdin".to_string(),
            })?;

        stdin
            .write_all(header.as_bytes())
            .and_then(|_| stdin.write_all(content.as_bytes()))
            .and_then(|_| stdin.flush())
            .map_err(|e| RefactorError::TransformFailed {
                message: format!("Failed to send LSP message: {}", e),
            })
    }

    fn read_response<R: DeserializeOwned>(&self, expected_id: i64) -> Result<R> {
        let mut stdout = self
            .stdout
            .lock()
            .map_err(|_| RefactorError::TransformFailed {
                message: "Failed to lock stdout".to_string(),
            })?;

        loop {
            // Read headers
            let mut header_line = String::new();
            let mut content_length = 0;

            loop {
                header_line.clear();
                stdout
                    .read_line(&mut header_line)
                    .map_err(|e| RefactorError::TransformFailed {
                        message: format!("Failed to read LSP header: {}", e),
                    })?;

                if header_line == "\r\n" || header_line.is_empty() {
                    break;
                }

                if let Some(len) = header_line.strip_prefix("Content-Length: ") {
                    content_length = len.trim().parse().unwrap_or(0);
                }
            }

            if content_length == 0 {
                return Err(RefactorError::TransformFailed {
                    message: "Invalid LSP response: no content length".to_string(),
                });
            }

            // Read content
            let mut content = vec![0u8; content_length];
            std::io::Read::read_exact(&mut *stdout, &mut content).map_err(|e| {
                RefactorError::TransformFailed {
                    message: format!("Failed to read LSP content: {}", e),
                }
            })?;

            let response: Value = serde_json::from_slice(&content)?;

            // Check if this is the response we're waiting for
            if let Some(id) = response.get("id").and_then(|v| v.as_i64()) {
                if id == expected_id {
                    if let Some(error) = response.get("error") {
                        return Err(RefactorError::TransformFailed {
                            message: format!("LSP error: {}", error),
                        });
                    }

                    let result = response.get("result").cloned().unwrap_or(Value::Null);
                    return serde_json::from_value(result).map_err(|e| {
                        RefactorError::TransformFailed {
                            message: format!("Failed to parse LSP response: {}", e),
                        }
                    });
                }
            }

            // Skip notifications and other messages
        }
    }

    fn detect_language_id(&self, path: &Path) -> String {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "ts" | "tsx" => "typescript",
                "js" | "jsx" => "javascript",
                "py" => "python",
                "go" => "go",
                "c" | "h" => "c",
                "cpp" | "cc" | "cxx" | "hpp" => "cpp",
                "java" => "java",
                "rb" => "ruby",
                _ => ext,
            })
            .unwrap_or("plaintext")
            .to_string()
    }

    fn convert_workspace_edit(&self, edit: &lsp_types::WorkspaceEdit) -> Result<WorkspaceEdit> {
        let mut result = WorkspaceEdit::new();

        if let Some(changes) = &edit.changes {
            for (uri, edits) in changes {
                let path = uri
                    .to_file_path()
                    .map_err(|_| RefactorError::TransformFailed {
                        message: format!("Invalid URI: {}", uri),
                    })?;

                let text_edits: Vec<TextEdit> = edits.iter().map(TextEdit::from_lsp).collect();
                result.add_edits(path, text_edits);
            }
        }

        if let Some(document_changes) = &edit.document_changes {
            let operations = match document_changes {
                lsp_types::DocumentChanges::Edits(edits) => edits
                    .iter()
                    .map(|e| lsp_types::DocumentChangeOperation::Edit(e.clone()))
                    .collect(),
                lsp_types::DocumentChanges::Operations(ops) => ops.clone(),
            };

            for change in operations {
                match change {
                    lsp_types::DocumentChangeOperation::Edit(text_doc_edit) => {
                        let path =
                            text_doc_edit
                                .text_document
                                .uri
                                .to_file_path()
                                .map_err(|_| RefactorError::TransformFailed {
                                    message: "Invalid URI in document edit".to_string(),
                                })?;

                        let text_edits: Vec<TextEdit> = text_doc_edit
                            .edits
                            .iter()
                            .map(|e| match e {
                                lsp_types::OneOf::Left(edit) => TextEdit::from_lsp(edit),
                                lsp_types::OneOf::Right(annotated) => {
                                    TextEdit::from_lsp(&annotated.text_edit)
                                }
                            })
                            .collect();

                        result.add_edits(path, text_edits);
                    }
                    _ => {} // Skip create/rename/delete operations for now
                }
            }
        }

        Ok(result)
    }

    fn convert_location(&self, loc: &lsp_types::Location) -> Result<crate::lsp::types::Location> {
        let path = loc
            .uri
            .to_file_path()
            .map_err(|_| RefactorError::TransformFailed {
                message: format!("Invalid URI: {}", loc.uri),
            })?;

        Ok(crate::lsp::types::Location::new(
            path,
            Range::from_lsp(loc.range),
        ))
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}
