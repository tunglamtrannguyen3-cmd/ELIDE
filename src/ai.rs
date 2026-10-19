// src/ai.rs
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use anyhow::{Result, Context};

pub struct AiConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
}

impl AiConfig {
    fn new() -> Self {
        Self {
            api_key: std::env::var("AI_API_KEY").unwrap_or_default(),
            api_url: std::env::var("AI_API_URL")
                .unwrap_or_else(|_| "https://api.deepseek.com/v1/chat/completions".to_string()),
            model: std::env::var("AI_MODEL")
                .unwrap_or_else(|_| "deepseek-coder".to_string()),
        }
    }
}

static AI_CONFIG: OnceLock<Mutex<AiConfig>> = OnceLock::new();

fn get_config() -> &'static Mutex<AiConfig> {
    AI_CONFIG.get_or_init(|| Mutex::new(AiConfig::new()))
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Serialize)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ChatChoice {
    pub message: ChatMessage,
}

#[derive(Deserialize, Debug)]
pub struct ChatResponse {
    pub choices: Vec<ChatChoice>,
}

/// Handles setting credentials via 'check <token> <url>' or auditing the repository via 'check <query>'
pub async fn run_check(workspace_root: &Path, input: &str) -> Result<String, String> {
    let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;

    // 1. Configure credentials on the fly if input contains a URL
    if input.contains("http://") || input.contains("https://") {
        let parts: Vec<&str> = input.split_whitespace().collect();
        if parts.len() >= 2 {
            config.api_key = parts[0].to_string();
            config.api_url = parts[1].to_string();
            return Ok(format!("🔑 Connected successfully!\nEndpoint: {}", config.api_url));
        } else {
            return Err("Usage for connection: check <token> <url>".to_string());
        }
    }

    if config.api_key.is_empty() && !config.api_url.contains("localhost") {
        return Err("Not connected! Run 'check <token> <url>' first.".to_string());
    }

    // 2. Gather workspace files using .gitignore-aware walk
    let mut code_context = String::new();
    let walker = WalkBuilder::new(workspace_root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for result in walker {
        if let Ok(entry) = result {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    if matches!(ext, "rs" | "toml" | "c" | "h" | "py" | "ts" | "js") {
                        if let Ok(content) = fs::read_to_string(path) {
                            let rel_path = path
                                .strip_prefix(workspace_root)
                                .unwrap_or(path)
                                .display();
                            code_context.push_str(&format!(
                                "\n--- FILE: {} ---\n{}\n",
                                rel_path, content
                            ));
                        }
                    }
                }
            }
        }
    }

    if code_context.is_empty() {
        return Err("No valid source files found in workspace to audit.".to_string());
    }

    let system_prompt = format!(
        "You are an expert code auditor embedded in the ELIDE editor (v2.1.1).\n\
         Here is the complete workspace codebase context:\n\n\
         {}\n\n\
         Analyze the codebase thoroughly and respond directly to the user's audit request.",
        code_context
    );

    let payload = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: input.to_string(),
            },
        ],
        temperature: Some(0.2),
        stream: None,
    };

    let client = reqwest::Client::new();
    let mut req = client.post(&config.api_url).json(&payload);

    if !config.api_key.is_empty() {
        req = req.bearer_auth(&config.api_key);
    }

    let res = req
        .send()
        .await
        .map_err(|e| format!("HTTP request failed: {}", e))?;

    if !res.status().is_success() {
        let err_body = res.text().await.unwrap_or_default();
        return Err(format!("API error response: {}", err_body));
    }

    let chat_res: ChatResponse = res
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

    chat_res
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .ok_or_else(|| "Received empty response from AI model.".to_string())
}

/// Handles multi-turn chat sessions in the AI chat pane
pub async fn run_chat_stream(root: &Path, history: Vec<(String, String)>) -> Result<String> {
    let config = get_config().lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;

    if config.api_key.is_empty() && !config.api_url.contains("localhost") {
        anyhow::bail!("AI API Key is missing. Set AI_API_KEY or run 'check <token> <url>' first.");
    }

    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: format!(
                "You are ELIDE AI, an expert systems programming assistant integrated into the ELIDE text editor. \
                The current workspace root is: {}. Help the user write, debug, and understand code.",
                root.display()
            ),
        }
    ];

    // Append full multi-turn conversation history
    for (role, content) in history {
        messages.push(ChatMessage { role, content });
    }

    let payload = ChatRequest {
        model: config.model.clone(),
        messages,
        temperature: Some(0.7),
        stream: Some(false),
    };

    let client = reqwest::Client::new();
    let mut req = client.post(&config.api_url).json(&payload);

    if !config.api_key.is_empty() {
        req = req.bearer_auth(&config.api_key);
    }

    let res = req
        .send()
        .await
        .context("Failed to connect to AI API endpoint")?;

    if !res.status().is_success() {
        let status = res.status();
        let err_text = res.text().await.unwrap_or_default();
        anyhow::bail!("API Error ({}): {}", status, err_text);
    }

    let chat_res: ChatResponse = res
        .json()
        .await
        .context("Failed to parse AI response JSON")?;

    let reply = chat_res
        .choices
        .first()
        .map(|c| c.message.content.clone())
        .unwrap_or_else(|| "No response generated.".to_string());

    Ok(reply)
}