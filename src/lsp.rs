// src/lsp.rs
use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::process::Stdio;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, Command};
use tokio::sync::mpsc;

// ==========================================
// 1. DIAGNOSTICS STRUCTURE
// ==========================================

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub line: usize,
    pub message: String,
    pub severity: Option<u8>,
}

// ==========================================
// 2. LSP WRITER (Background Stdin Pipe Manager)
// ==========================================

#[derive(Clone)]
pub struct LspWriter {
    tx: mpsc::UnboundedSender<Value>,
}

impl LspWriter {
    /// Spawns a dedicated background task owning `stdin` so the pipe is never dropped.
    pub fn spawn(mut stdin: tokio::process::ChildStdin) -> Self {
        let (tx, mut rx) = mpsc::unbounded_channel::<Value>();

        tokio::spawn(async move {
            while let Some(msg) = rx.recv().await {
                let msg_str = msg.to_string();
                let payload = format!("Content-Length: {}\r\n\r\n{}", msg_str.len(), msg_str);

                if stdin.write_all(payload.as_bytes()).await.is_err() || stdin.flush().await.is_err() {
                    break; // Process terminated or pipe closed
                }
            }
        });

        Self { tx }
    }

    /// Non-blocking method to queue JSON-RPC payloads to the LSP server.
    pub fn send(&self, msg: Value) -> Result<()> {
        self.tx.send(msg)?;
        Ok(())
    }
}

// ==========================================
// 3. LSP CLIENT & MANAGEMENT
// ==========================================

pub struct LspClient {
    pub server_name: String,
    pub language_id: String,
    child: Option<Child>,
    writer: Option<LspWriter>,
    next_id: Arc<AtomicU64>,
    opened_uris: HashSet<String>,
}

impl LspClient {
    pub fn new() -> Self {
        Self {
            server_name: "Disconnected".to_string(),
            language_id: "plaintext".to_string(),
            child: None,
            writer: None,
            next_id: Arc::new(AtomicU64::new(1)),
            opened_uris: HashSet::new(),
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
        // 1. Gracefully shutdown old process if one exists
        if let Some(ref writer) = self.writer {
            let shutdown_id = self.next_id.fetch_add(1, Ordering::SeqCst);
            let _ = writer.send(json!({
                "jsonrpc": "2.0",
                "id": shutdown_id,
                "method": "shutdown",
                "params": null
            }));
            let _ = writer.send(json!({
                "jsonrpc": "2.0",
                "method": "exit",
                "params": null
            }));
        }

        if let Some(mut old_child) = self.child.take() {
            tokio::select! {
                _ = old_child.wait() => {}
                _ = tokio::time::sleep(Duration::from_millis(150)) => {
                    let _ = old_child.kill().await;
                }
            }
        }

        self.opened_uris.clear();

       // 2. Safely tokenize command string (respecting quotes & spaces)
        let tokens = parse_cli_tokens(lsp_cmd);
        let program = tokens
            .first()
            .cloned()
            .ok_or_else(|| anyhow!("Empty LSP command provided"))?;
        let args: Vec<String> = tokens.into_iter().skip(1).collect();

        // 3. Spawn LSP process (Redirect stderr to log file for debugging)
        let stderr_file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open("elide_debug.log")
            .unwrap_or_else(|_| std::fs::File::create("/dev/null").unwrap());

        let mut child = Command::new(&program) // Pass a reference here!
            .args(&args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::from(stderr_file))
            .spawn()
            .map_err(|e| {
                self.server_name = format!("Failed to launch {}", program); // program is still valid here
                anyhow!("Failed to spawn LSP process '{}': {}", lsp_cmd, e)
            })?;

        self.server_name = lsp_cmd.to_string();
        self.language_id = language_id.to_string();
        
        let active_lang = language_id.to_string();

        let stdin = child.stdin.take().expect("Failed to open stdin");
        let stdout = child.stdout.take().expect("Failed to open stdout");

        self.child = Some(child);

        // 4. Spawn background writer
        let writer = LspWriter::spawn(stdin);
        self.writer = Some(writer.clone());

        let root_uri = get_file_uri(workspace_dir);
        let init_id = self.next_id.fetch_add(1, Ordering::SeqCst);

        // --- STEP 1: Send initialize request ---
        let init_req = json!({
            "jsonrpc": "2.0",
            "id": init_id,
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

        // --- STEP 2: Wait for initialization response ---
        let mut reader = BufReader::new(stdout);
        loop {
            let msg = read_lsp_message(&mut reader).await?;
            if let Ok(json) = serde_json::from_slice::<Value>(&msg) {
                if json.get("id").and_then(|id| id.as_u64()) == Some(init_id) {
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

        // --- STEP 3.5: Automatically open all matching workspace files without duplicates ---
        let workspace_path = std::path::PathBuf::from(workspace_dir);
        let target_lang = active_lang.clone();
        let writer_clone = writer.clone();
        
        tokio::spawn(async move {
            let walker = ignore::WalkBuilder::new(&workspace_path)
                .hidden(true)
                .git_ignore(true)
                .build();

            for entry in walker.flatten() {
                let path = entry.path();
                if path.is_file() {
                    let path_str = path.to_string_lossy();
                    if path_str.contains("/target/") || path_str.contains("/node_modules/") || path_str.contains("/.git/") {
                        continue;
                    }

                    if get_language_id(&path_str) == target_lang {
                        if let Ok(content) = std::fs::read_to_string(path) {
                            let file_uri = get_file_uri(&path_str);
                            let did_open_req = json!({
                                "jsonrpc": "2.0",
                                "method": "textDocument/didOpen",
                                "params": {
                                    "textDocument": {
                                        "uri": file_uri,
                                        "languageId": target_lang,
                                        "version": 1,
                                        "text": content
                                    }
                                }
                            });
                            let _ = writer_clone.send(did_open_req);
                        }
                    }
                }
            }
        });

        // --- STEP 4: Start background diagnostic listener ---
        tokio::spawn(async move {
            while let Ok(body) = read_lsp_message(&mut reader).await {
                if let Ok(json) = serde_json::from_slice::<Value>(&body) {
                    if json.get("method").and_then(|m| m.as_str()) == Some("textDocument/publishDiagnostics") {
                        if let Some(uri_str) = json.pointer("/params/uri").and_then(|u| u.as_str()) {
                            let file_path = match url::Url::parse(uri_str) {
                                Ok(url) => url.to_file_path().unwrap_or_else(|_| std::path::PathBuf::from(uri_str)),
                                Err(_) => std::path::PathBuf::from(uri_str.strip_prefix("file://").unwrap_or(uri_str)),
                            };
                            
                            let path_lossy = file_path.to_string_lossy();
                            if get_language_id(&path_lossy) != active_lang {
                                continue;
                            }

                            let file_uri = uri_str.to_string();

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
        });
        Ok(())
    }

    /// Called when the user opens a specific file in the editor (guarded against duplicate opens)
    pub fn open_file(&mut self, filename: &str, text: &str) -> Result<()> {
        if get_language_id(filename) != self.language_id {
            return Ok(());
        }

        let uri = get_file_uri(filename);
        if self.opened_uris.contains(&uri) {
            // Already opened via workspace scan or prior open; notify change instead of duplicate open
            return self.notify_change(filename, 1, text);
        }

        self.opened_uris.insert(uri.clone());

        let did_open_req = json!({
            "jsonrpc": "2.0",
            "method": "textDocument/didOpen",
            "params": {
                "textDocument": {
                    "uri": uri,
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
            return Ok(());
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

impl Drop for LspClient {
    fn drop(&mut self) {
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
        }
    }
}

impl Default for LspClient {
    fn default() -> Self {
        Self::new()
    }
}

// ==========================================
// 4. DIRECTORY SCANNING & LANGUAGE IDENTIFICATION
// ==========================================

pub fn detect_workspace_language(dir: &str) -> Option<&'static str> {
    let path = Path::new(dir);
    let mut ext_counts: HashMap<&'static str, usize> = HashMap::new();

    fn visit_dirs(dir: &Path, counts: &mut HashMap<&'static str, usize>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let name = path.file_name().unwrap_or_default().to_string_lossy();
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

    ext_counts
        .into_iter()
        .max_by_key(|&(_, count)| count)
        .map(|(lang, _)| lang)
}

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
    let path = Path::new(filename);
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(filename)
        .to_lowercase();
    
    if file_name.ends_with("dockerfile") || file_name.starts_with("dockerfile.") {
        return "dockerfile";
    }
    if file_name == "makefile" || file_name.ends_with(".mk") {
        return "makefile";
    }
    if file_name == "jenkinsfile" {
        return "jenkinsfile";
    }
    if file_name == "cmakelists.txt" {
        return "cmake";
    }

    let ext = file_name.rsplit('.').next().unwrap_or("");
    match ext {
        "rs" => "rust",
        "zig" => "zig",
        "c" => "c",
        "cpp" | "cxx" | "cc" | "h" | "hpp" => "cpp",
        "go" => "go",
        "d" | "di" => "d",
        "nim" => "nim",
        "v" => "v",
        "asm" | "s" | "S" => "assembly",
        "gd" => "gdscript",
        "cs" => "csharp",
        "glsl" | "vert" | "frag" | "geom" | "comp" => "glsl",
        "hlsl" | "fx" | "hlsli" => "hlsl",
        "wgsl" => "wgsl",
        "shader" => "unity-shader",
        "brres" | "gsh" => "graphics-resource", 
        "java" => "java",
        "kt" | "kts" => "kotlin",
        "scala" | "sc" => "scala",
        "swift" => "swift",
        "oc" | "m" | "mm" => "objective-c",
        "adb" | "ads" | "ada" | "gpr" => "ada",
        "py" | "pyw" => "python",
        "rb" | "rbw" | "gemspec" => "ruby",
        "php" => "php",
        "pl" | "pm" | "t" => "perl",
        "lua" => "lua",
        "tcl" => "tcl",
        "js" | "jsx" | "mjs" | "cjs" => "javascript",
        "ts" | "tsx" | "mts" | "cts" => "typescript",
        "html" | "htm" => "html",
        "css" => "css",
        "scss" => "scss",
        "less" => "less",
        "svelte" => "svelte",
        "vue" => "vue",
        "astro" => "astro",
        "graphql" | "gql" => "graphql",
        "hs" | "lhs" => "haskell",
        "ml" | "mli" => "ocaml",
        "ex" | "exs" => "elixir",
        "erl" | "hrl" => "erlang",
        "clj" | "cljs" | "edn" => "clojure",
        "gleam" => "gleam",
        "jl" => "julia",
        "r" | "rmd" => "r",
        "sol" => "solidity",
        "move" => "move",
        "sh" | "bash" | "zsh" => "bash",
        "ps1" | "psm1" => "powershell",
        "json" | "jsonc" => "json",
        "yaml" | "yml" => "yaml",
        "toml" => "toml",
        "ini" | "cfg" | "conf" => "ini",
        "xml" | "xsd" | "xsl" => "xml",
        "tf" | "tfvars" => "terraform",
        "proto" => "protobuf",
        "env" => "dotenv",
        "cmake" => "cmake",
        "nix" => "nix",
        "dockerfile" | "docker" => "dockerfile",
        "wit" => "wit",
        "sql" => "sql",
        "csv" => "csv",
        "tsv" => "tsv",
        "md" | "markdown" => "markdown",
        "tex" | "latex" => "latex",
        "rst" => "restructuredtext",
        "asciidoc" | "adoc" => "asciidoc",
        "org" => "org",
        "diff" | "patch" => "diff",
        _ => "plaintext",
    }
}

/// Helper to split command strings while respecting quoted sentences and paths
fn parse_cli_tokens(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;

    for c in input.chars() {
        match c {
            '"' | '\'' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(current.clone());
                    current.clear();
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens
}