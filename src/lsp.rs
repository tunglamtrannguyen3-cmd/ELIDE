use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc;
use crate::diagnostics::{Diagnostic, DiagnosticSeverity};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LspStatus {
    Disconnected,
    Connecting,
    Connected,
    MissingBinary(String),
    Error(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonRpcMessage {
    jsonrpc: String,
    method: Option<String>,
    params: Option<serde_json::Value>,
}

pub struct LspClient {
    pub server_name: String,
    pub status: LspStatus,
    process: Option<Child>,
    stdin: Option<ChildStdin>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: String::new(),
            status: LspStatus::Disconnected,
            process: None,
            stdin: None,
        }
    }

    pub fn get_server_command(filename: &str) -> Option<(&'static str, &'static str)> {
        let ext = Path::new(filename).extension()?.to_str()?.to_lowercase();
        match ext.as_str() {
            "rs" => Some(("rust-analyzer", "rust-analyzer")),
            "c" | "h" | "cpp" | "hpp" => Some(("clangd", "clangd")),
            "adb" | "ads" | "ada" => Some(("ada_language_server", "ada_language_server")),
            _ => None,
        }
    }

    pub async fn start(
        &mut self,
        filename: &str,
        diag_tx: mpsc::UnboundedSender<Vec<Diagnostic>>,
    ) -> Result<()> {
        let (name, binary) = match Self::get_server_command(filename) {
            Some(cmd) => cmd,
            None => {
                self.status = LspStatus::Disconnected;
                return Ok(());
            }
        };

        self.server_name = name.to_string();
        self.status = LspStatus::Connecting;

        let mut child = match Command::new(binary)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
        {
            Ok(c) => c,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let msg = format!("'{}' missing in PATH.", binary);
                self.status = LspStatus::MissingBinary(msg);
                return Ok(());
            }
            Err(e) => {
                let err_msg = format!("Failed to spawn {}: {}", binary, e);
                self.status = LspStatus::Error(err_msg);
                return Ok(());
            }
        };

        self.stdin = child.stdin.take();
        let stdout = child.stdout.take();
        self.process = Some(child);

        if let Some(stdout) = stdout {
            tokio::spawn(async move {
                let _ = Self::read_stdout_loop(stdout, diag_tx).await;
            });
        }

        self.status = LspStatus::Connected;
        
        let init_payload = r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"capabilities":{}}}"#;
        self.send_rpc_message(init_payload).await?;

        Ok(())
    }

    pub async fn notify_did_open(&mut self, filename: &str, file_content: &str) -> Result<()> {
        if self.status != LspStatus::Connected {
            return Ok(());
        }

        let abs_path = std::fs::canonicalize(filename)
            .unwrap_or_else(|_| std::path::PathBuf::from(filename));
        let uri = format!("file://{}", abs_path.display());
        
        let ext = std::path::Path::new(filename)
            .extension()
            .and_then(|s| s.to_str())
            .unwrap_or("");
            
        let lang_id = match ext {
            "rs" => "rust",
            "c" | "h" => "c",
            "cpp" | "hpp" => "cpp",
            "adb" | "ads" | "ada" => "ada",
            _ => ext,
        };

        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
                    "languageId": lang_id,
                    "version": 1,
                    "text": file_content
                }
            }
        });

        self.send_rpc_message(&payload.to_string()).await
    }

    pub async fn send_rpc_message(&mut self, payload: &str) -> Result<()> {
        if let Some(ref mut stdin) = self.stdin {
            let header = format!("Content-Length: {}\r\n\r\n", payload.len());
            stdin.write_all(header.as_bytes()).await?;
            stdin.write_all(payload.as_bytes()).await?;
            stdin.flush().await?;
        }
        Ok(())
    }

    async fn read_stdout_loop(
        stdout: ChildStdout,
        diag_tx: mpsc::UnboundedSender<Vec<Diagnostic>>,
    ) -> Result<()> {
        let mut reader = BufReader::new(stdout);
        let mut line = String::new();

        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break;
            }

            if line.starts_with("Content-Length: ") {
                let len_str = line.trim_start_matches("Content-Length: ").trim();
                if let Ok(content_len) = len_str.parse::<usize>() {
                    line.clear();
                    reader.read_line(&mut line).await?;

                    let mut body_buf = vec![0u8; content_len];
                    tokio::io::AsyncReadExt::read_exact(&mut reader, &mut body_buf).await?;

                    if let Ok(msg) = serde_json::from_slice::<JsonRpcMessage>(&body_buf) {
                        if msg.method.as_deref() == Some("textDocument/publishDiagnostics") {
                            if let Some(params) = msg.params {
                                let parsed_diags = Self::parse_diagnostics_params(&params);
                                let _ = diag_tx.send(parsed_diags);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn parse_diagnostics_params(params: &serde_json::Value) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        if let Some(diagnostics_list) = params.get("diagnostics").and_then(|d| d.as_array()) {
            for item in diagnostics_list {
                let line = item.pointer("/range/start/line")
                    .and_then(|l| l.as_u64())
                    .unwrap_or(0) as usize + 1;

                let message = item.get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("Unknown issue")
                    .to_string();

                let severity = match item.get("severity").and_then(|s| s.as_u64()) {
                    Some(1) => DiagnosticSeverity::Error,
                    Some(2) => DiagnosticSeverity::Warning,
                    Some(3) => DiagnosticSeverity::Info,
                    _ => DiagnosticSeverity::Hint,
                };

                diags.push(Diagnostic { line, message, severity });
            }
        }
        diags
    }
}
