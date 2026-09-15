use anyhow::Result;
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader}; // AsyncWriteExt removed
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use crate::diagnostics::Diagnostic;

pub struct LspClient {
    pub server_name: String,
    _child: Option<Child>,
    writer: Option<crate::lsp_writer::LspWriter>,
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

        self.server_name = lsp_cmd.to_string();

        let mut child = match Command::new(lsp_cmd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(_) => {
                self.server_name = format!("Failed to launch {}", lsp_cmd);
                return Ok(());
            }
        };

        let stdin = child.stdin.take().expect("Failed to open stdin"); // Removed 'mut'
        let stdout = child.stdout.take().expect("Failed to open stdout");

        self._child = Some(child);

        // --- NEW: Spawn the persistent background writer ---
        let writer = crate::lsp_writer::LspWriter::spawn(stdin);
        self.writer = Some(writer.clone());

        let file_path = std::fs::canonicalize(filename).unwrap_or_else(|_| std::path::PathBuf::from(filename));
        let file_uri = format!("file://{}", file_path.display());
        let current_dir = std::env::current_dir().unwrap_or_default();
        let root_uri = format!("file://{}", current_dir.display());

        let ext = filename.split('.').last().unwrap_or("");
        let language_id = match ext {
            "rs" => "rust",
            "c" => "c",
            "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
            "py" => "python",
            "adb" | "ads" | "ada" => "ada",
            "js" => "javascript",
            "ts" => "typescript",
            _ => "plaintext",
        };

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
        writer.send(init_req)?; // --- NEW

        // --- STEP 2: Wait for the LSP to reply ---
        let mut reader = BufReader::new(stdout);
        let mut len = 0;
        loop {
            let mut header_line = String::new();
            if reader.read_line(&mut header_line).await.unwrap_or(0) == 0 {
                return Err(anyhow::anyhow!("LSP stream closed prematurely")); 
            }
            let header_line = header_line.trim();
            if header_line.is_empty() { break; }
            if header_line.starts_with("Content-Length:") {
                if let Ok(l) = header_line[15..].trim().parse::<usize>() {
                    len = l;
                }
            }
        }

        if len > 0 {
            let mut body = vec![0; len];
            reader.read_exact(&mut body).await?; // Consume the InitializeResult
        }

        // --- STEP 3: Safe to send notifications now! ---
        let initialized_notif = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        writer.send(initialized_notif)?; // --- NEW

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
        writer.send(did_open_req)?; // --- NEW

        // --- STEP 4: Start continuous background diagnostic listener ---
        tokio::spawn(async move {
            loop {
                let mut len = 0;
                loop {
                    let mut header_line = String::new();
                    if reader.read_line(&mut header_line).await.unwrap_or(0) == 0 { return; }
                    let header_line = header_line.trim();
                    if header_line.is_empty() { break; }
                    if header_line.starts_with("Content-Length:") {
                        if let Ok(l) = header_line[15..].trim().parse::<usize>() { len = l; }
                    }
                }

                if len > 0 {
                    let mut body = vec![0; len];
                    if reader.read_exact(&mut body).await.is_ok() {
                        if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&body) {
                            if json["method"] == "textDocument/publishDiagnostics" {
                                if let Some(diags_array) = json["params"]["diagnostics"].as_array() {
                                    let mut parsed_diags = Vec::new();
                                    for d in diags_array {
                                        let line = d["range"]["start"]["line"].as_u64().unwrap_or(0) as usize + 1;
                                        let message = d["message"].as_str().unwrap_or("Unknown error").to_string();
                                        parsed_diags.push(Diagnostic { line, message });
                                    }
                                    let _ = diag_tx.send(parsed_diags);
                                }
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    // --- NEW: Handle document updates from main.rs ---
    pub fn notify_change(&self, filename: &str, version: u32, text: &str) -> Result<()> {
        let file_path = std::fs::canonicalize(filename).unwrap_or_else(|_| std::path::PathBuf::from(filename));
        let file_uri = format!("file://{}", file_path.display());

        let did_change_req = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didChange",
            "params": {
                "textDocument": {
                    "uri": file_uri,
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