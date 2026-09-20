// src/lsp.rs
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

use crate::diagnostics::Diagnostic;
use crate::lsp_writer::LspWriter;

pub struct LspClient {
    pub server_name: String,
    pub language_id: String,
    _child: Option<Child>,
    writer: Option<LspWriter>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: "Disconnected".to_string(),
            language_id: "plaintext".to_string(),
            _child: None,
            writer: None,
        }
    }

    /// Initializes the LSP at the workspace (directory) level for a specific language.
    pub async fn init_workspace(
        &mut self,
        lsp_cmd: &str,
        language_id: &str,
        workspace_dir: &str,
        diag_tx: mpsc::UnboundedSender<(String, Vec<Diagnostic>)>,
    ) -> Result<()> {
        // 1. Kill old process if one exists
        if let Some(mut old_child) = self._child.take() {
            let _ = old_child.kill().await;
        }

        // 2. Safely split the command string into executable and arguments
        let mut parts = lsp_cmd.split_whitespace();
        let program = parts.next().ok_or_else(|| anyhow!("Empty LSP command provided"))?;
        let args: Vec<&str> = parts.collect();

        // 3. Spawn LSP process
        let mut child = Command::new(program)
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| {
                self.server_name = format!("Failed to launch {}", program);
                anyhow!("Failed to spawn LSP process '{}': {}", lsp_cmd, e)
            })?;

        self.server_name = lsp_cmd.to_string();
        self.language_id = language_id.to_string();
        
        let active_lang = language_id.to_string();

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        self._child = Some(child);

        // 4. Spawn background writer
        let writer = LspWriter::spawn(stdin);
        self.writer = Some(writer.clone());

        let root_uri = get_file_uri(workspace_dir);

        // --- STEP 1: Send the initialize request ---
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "processId": std::process::id(),
                "rootUri": root_uri,
                "workspaceFolders": [{
                    "uri": &root_uri,
                    "name": "workspace"
                }],
                "capabilities": {
                    "workspace": {
                        "workspaceFolders": true
                    },
                    "textDocument": {
                        "synchronization": {
                            "dynamicRegistration": false,
                            "willSave": false,
                            "willSaveWaitUntil": false,
                            "didSave": false
                        },
                        "publishDiagnostics": {
                            "relatedInformation": true
                        }
                    }
                }
            }
        });
        writer.send(init_req)?;

        // --- STEP 2: Wait specifically for the initialization response ---
        let mut reader = BufReader::new(stdout);
        loop {
            let msg = read_lsp_message(&mut reader).await?;
            if let Ok(json) = serde_json::from_slice::<Value>(&msg) {
                if json.get("id").and_then(|id| id.as_u64()) == Some(1) {
                    break;
                }
            }
        }

        // --- STEP 3: Send initialized notification ---
        let initialized_notif = json!({
            "jsonrpc": "2.0",
            "method": "initialized",
            "params": {}
        });
        writer.send(initialized_notif)?;

        // --- STEP 4: Start background diagnostic listener ---
        tokio::spawn(async move {
            loop {
                match read_lsp_message(&mut reader).await {
                    Ok(body) => {
                        if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                            if json.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
                                if let Some(uri) = json.pointer("/params/uri").and_then(|u| u.as_str()) {
                                    
                                    // Extract path and filter by target language
                                    let path = uri.strip_prefix("file://").unwrap_or(uri);
                                    if get_language_id(path) != active_lang {
                                        continue; // Silently drop diagnostics for other languages
                                    }

                                    let file_uri = uri.to_string();

                                    if let Some(diags_array) = json.pointer("/params/diagnostics").and_then(|d| d.as_array()) {
                                        let parsed_diags: Vec<Diagnostic> = diags_array.iter().map(|d| {
                                            let line = d.pointer("/range/start/line")
                                                .and_then(|l| l.as_u64())
                                                .unwrap_or(0) as usize + 1;
                                            
                                            let message = d.get("message")
                                                .and_then(|m| m.as_str())
                                                .unwrap_or("Unknown error")
                                                .to_string();
                                                
                                            let severity = d.get("severity")
                                                .and_then(|s| s.as_u64())
                                                .map(|n| n as u8);
                                                
                                            Diagnostic { 
                                                line, 
                                                message, 
                                                severity 
                                            }
                                        }).collect();

                                        let _ = diag_tx.send((file_uri, parsed_diags));
                                    }
                                }
                            }
                        }
                    }
                    Err(_) => break, // Stream closed or malformed
                }
            }
        });

        Ok(())
    }

    /// Called when the user opens a specific file in the editor
    pub fn open_file(&self, filename: &str, text: &str) -> Result<()> {
        if get_language_id(filename) != self.language_id {
            return Ok(()); // Do not send unrelated files to the LSP
        }

        let did_open_req = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": get_file_uri(filename),
                    "languageId": get_language_id(filename),
                    "version": 1,
                    "text": text
                }
            }
        });

        if let Some(writer) = &self.writer {
            writer.send(did_open_req)?;
        }
        Ok(())
    }

    /// Called when the user types/modifies the file
    pub fn notify_change(&self, filename: &str, version: u32, text: &str) -> Result<()> {
        if get_language_id(filename) != self.language_id {
            return Ok(()); // Do not send unrelated file changes to the LSP
        }

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

// --- Directory Scanning Logic ---

/// Scans the directory to find the most used language. 
/// It skips build/dependency directories to stay fast.
pub fn detect_workspace_language(dir: &str) -> Option<&'static str> {
    let path = Path::new(dir);
    let mut ext_counts: HashMap<&'static str, usize> = HashMap::new();

    fn visit_dirs(dir: &Path, counts: &mut HashMap<&'static str, usize>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
                    // Skip hidden dirs and common heavy folders
                    if name.starts_with('.') || name == "target" || name == "node_modules" || name == "build" {
                        continue;
                    }
                    visit_dirs(&path, counts);
                } else if path.is_file() {
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    let dummy_file = format!("f.{}", ext);
                    let lang = get_language_id(&dummy_file);
                    if lang != "plaintext" {
                        *counts.entry(lang).or_insert(0) += 1;
                    }
                }
            }
        }
    }

    visit_dirs(path, &mut ext_counts);

    // Return the language with the highest count
    ext_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, _)| lang)
}

// --- Helper Functions ---

async fn read_lsp_message<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> Result<Vec<u8>> {
    let mut len = 0;
    loop {
        let mut header_line = String::new();
        if reader.read_line(&mut header_line).await? == 0 {
            return Err(anyhow!("LSP stream closed prematurely"));
        }
        
        let header_line = header_line.trim();
        if header_line.is_empty() {
            break;
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

fn get_file_uri(filename: &str) -> String {
    let file_path = std::fs::canonicalize(filename)
        .unwrap_or_else(|_| std::env::current_dir().unwrap_or_default().join(filename));
    url::Url::from_file_path(file_path)
        .map(|u| u.to_string())
        .unwrap_or_else(|_| format!("file://{}", filename))
}

pub fn get_language_id(filename: &str) -> &'static str {
    let lowercase_name = filename.to_lowercase();
    if lowercase_name.ends_with("dockerfile") {
        return "dockerfile";
    }

    let ext = filename.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "c" => "c",
        "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
        "py" => "python",
        "go" => "go",
        "java" => "java",
        "cs" => "csharp",
        "js" | "jsx" => "javascript",
        "ts" | "tsx" => "typescript",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "php" => "php",
        "rb" => "ruby",
        "lua" => "lua",
        "sh" | "bash" => "bash",
        "dart" => "dart",
        "svelte" => "svelte",
        "vue" => "vue",
        "hs" => "haskell",
        "ml" | "mli" => "ocaml",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "json" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "md" | "markdown" => "markdown",
        "tex" | "latex" => "latex",
        "dockerfile" | "docker" => "dockerfile",
        "sql" => "sql",
        "adb" | "ads" | "ada" => "ada",
        "swift" => "swift",
        "nim" => "nim",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        _ => "plaintext",
    }
}