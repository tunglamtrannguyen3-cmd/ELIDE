use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::path::PathBuf;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::diagnostics::Diagnostic;
use crate::lsp_writer::LspWriter;

pub struct LspClient {
    pub server_name: String,
    _child: Option<Child>,
    writer: Option<LspWriter>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: "Disconnected".to_string(),
            _child: None,
            writer: None,
        }
    }

    pub async fn start(
        &mut self,
        lsp_cmd: &str,
        filename: &str,
        diag_tx: mpsc::UnboundedSender<Vec<Diagnostic>>,
    ) -> Result<()> {
        // 1. Kill the old LSP process if one is currently running
        if let Some(mut old_child) = self._child.take() {
            let _ = old_child.kill().await;
        }

        // 2. Spawn the new LSP process
        let mut child = Command::new(lsp_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                self.server_name = format!("Failed to launch {}", lsp_cmd);
                anyhow!("Failed to spawn LSP process '{}': {}", lsp_cmd, e)
            })?;

        self.server_name = lsp_cmd.to_string();

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        self._child = Some(child);

        // 3. Spawn the persistent background writer
        let writer = LspWriter::spawn(stdin);
        self.writer = Some(writer.clone());

        let file_uri = get_file_uri(filename);
        let root_uri = get_file_uri(&std::env::current_dir().unwrap_or_default().to_string_lossy());
        let language_id = get_language_id(filename);

        // --- STEP 1: Send the initialize request ---
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
                "capabilities": {
                    "textDocument": {
                        "publishDiagnostics": {
                            "relatedInformation": true
                        }
                    }
                }
            }
        });
        writer.send(init_req)?;

        // --- STEP 2: Wait for the LSP to reply ---
        let mut reader = BufReader::new(stdout);
        let _initialize_result = read_lsp_message(&mut reader).await?;

        // --- STEP 3: Safe to send notifications now ---
        let initialized_notif = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        writer.send(initialized_notif)?;

        let did_open_req = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": file_uri,
                    "languageId": language_id,
                    "version": 1,
                    "text": std::fs::read_to_string(filename).unwrap_or_default()
                }
            }
        });
        writer.send(did_open_req)?;

        // --- STEP 4: Start continuous background diagnostic listener ---
        tokio::spawn(async move {
            loop {
                match read_lsp_message(&mut reader).await {
                    Ok(body) => {
                        if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                            if json.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
                                if let Some(diags_array) = json.pointer("/params/diagnostics").and_then(|d| d.as_array()) {
                                    let parsed_diags: Vec<Diagnostic> = diags_array.iter().map(|d| {
                                        let line = d.pointer("/range/start/line")
                                            .and_then(|l| l.as_u64())
                                            .unwrap_or(0) as usize + 1;
                                        
                                        let message = d.get("message")
                                            .and_then(|m| m.as_str())
                                            .unwrap_or("Unknown error")
                                            .to_string();
                                            
                                        Diagnostic { line, message }
                                    }).collect();

                                    let _ = diag_tx.send(parsed_diags);
                                }
                            }
                        }
                    }
                    Err(_) => break, // Stream closed or malformed, exit task cleanly
                }
            }
        });

        Ok(())
    }

    pub fn notify_change(&self, filename: &str, version: u32, text: &str) -> Result<()> {
        let did_change_req = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": get_file_uri(filename),
                    "version": version
                },
                "contentChanges": [
                    { "text": text }
                ]
            }
        });

        if let Some(writer) = &self.writer {
            writer.send(did_change_req)?;
        }
        
        Ok(())
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}

// --- Helper Functions ---

/// Reads the next JSON-RPC payload from the LSP stream by parsing the `Content-Length` header.
async fn read_lsp_message<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len = 0;
    
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).await? == 0 {
            return Err(anyhow!("LSP stream closed prematurely"));
        }
        
        let header_line = header_line.trim();
        if header_line.is_empty() {
            break; // Empty line signifies the end of headers
        }
        
        if let Some(len_str) = header_line.strip_prefix("Content-Length:") {
            len = len_str.trim().parse::<usize>()?;
        }
    }

    if len == 0 {
        return Err(anyhow!("Missing or invalid Content-Length header"));
    }

    let mut body = vec![0; len];
    reader.read_exact(&mut body).await?;
    
    Ok(body)
}

/// Converts a local file path into an LSP-compatible file URI.
fn get_file_uri(filename: &str) -> String {
    let file_path = std::fs::canonicalize(filename).unwrap_or_else(|_| PathBuf::from(filename));
    format!("file://{}", file_path.display())
}

/// Resolves the LSP language ID based on the file extension.
fn get_language_id(filename: &str) -> &'static str {
    match filename.split('.').last().unwrap_or("") {
        "rs" => "rust",
        "c" => "c",
        "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
        "py" => "python",
        "adb" | "ads" | "ada" => "ada",
        "js" => "javascript",
        "ts" => "typescript",
        _ => "plaintext",
    }
}