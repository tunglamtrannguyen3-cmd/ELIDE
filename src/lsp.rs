use anyhow::Result;
use serde_json::json;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;
use crate::diagnostics::Diagnostic;

pub struct LspClient {
    pub server_name: String,
    _child: Option<Child>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: "Disconnected".to_string(),
            _child: None,
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

        let mut stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        // Save the new process so we can kill it next time we switch
        self._child = Some(child);

        let file_path = std::fs::canonicalize(filename).unwrap_or_else(|_| std::path::PathBuf::from(filename));
        let file_uri = format!("file://{}", file_path.display());
        let current_dir = std::env::current_dir().unwrap_or_default();
        let root_uri = format!("file://{}", current_dir.display());

        // 2. Map standard file extensions to LSP Language IDs
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
        send_message(&mut stdin, init_req).await?;

        let initialized_notif = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        send_message(&mut stdin, initialized_notif).await?;

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
        send_message(&mut stdin, did_open_req).await?;

        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            loop {
                let mut len = 0;
                loop {
                    let mut header_line = String::new();
                    if reader.read_line(&mut header_line).await.unwrap_or(0) == 0 {
                        return; 
                    }
                    let header_line = header_line.trim();
                    if header_line.is_empty() {
                        break; 
                    }
                    if header_line.starts_with("Content-Length:") {
                        if let Ok(l) = header_line[15..].trim().parse::<usize>() {
                            len = l;
                        }
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
}

async fn send_message(stdin: &mut tokio::process::ChildStdin, msg: serde_json::Value) -> Result<()> {
    let msg_str = msg.to_string();
    let payload = format!("Content-Length: {}\r\n\r\n{}", msg_str.len(), msg_str);
    stdin.write_all(payload.as_bytes()).await?;
    stdin.flush().await?;
    Ok(())
}