// src/ai.rs
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;
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
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1/chat/completions".to_string()),
            model: std::env::var("AI_MODEL")
                .unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string()), // Defaulted to Llama 3.3 70B
        }
    }
}

static AI_CONFIG: OnceLock<Mutex<AiConfig>> = OnceLock::new();
static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();

fn get_config() -> &'static Mutex<AiConfig> {
    AI_CONFIG.get_or_init(|| Mutex::new(AiConfig::new()))
}

fn get_http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .unwrap_or_default()
    })
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

pub async fn run_check(workspace_root: &Path, input: &str) -> Result<String, String> {
    let input_trimmed = input.trim();

    // 1. SETUP STEP: Detect connection command containing an endpoint URL
    if input_trimmed.contains("http://") || input_trimmed.contains("https://") {
        let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
        if parts.len() >= 2 {
            let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
            config.api_key = parts[0].to_string();
            config.api_url = parts[1].to_string();

            // Optional 3rd argument allows overriding the model if desired
            if parts.len() >= 3 {
                config.model = parts[2].to_string();
            }

            return Ok(format!(
                "🔑 Connection saved successfully!\n- Endpoint: {}\n- Model: {}\n\nYou can now run 'check <your prompt>' to audit your workspace.",
                config.api_url, config.model
            ));
        } else {
            return Err("Usage for connection: check <token> <url> [model]".to_string());
        }
    }

    // 2. READ CONFIGURATION: Verify credentials before handling prompt
    let (api_key, api_url, model) = {
        let config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
        if config.api_key.is_empty() && !config.api_url.contains("localhost") {
            return Err("❌ AI Not Connected!\nPlease configure your credentials first using:\ncheck <token> <url> [model]".to_string());
        }
        (config.api_key.clone(), config.api_url.clone(), config.model.clone())
    }; // Lock safely dropped before await!

    // 3. EXECUTE PROMPT: Gather workspace files using gitignore-aware walker
    let mut code_context = String::new();
    let walker = WalkBuilder::new(workspace_root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
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

    if code_context.is_empty() {
        return Err("No valid source files found in workspace to audit.".to_string());
    }

    let system_prompt = format!(
        "You are ELIDE AI, a friendly and expert systems programming co-pilot integrated into the ELIDE text editor (v2.3.1).\n\
         You have access to the user's complete workspace codebase context below:\n\n\
         {}\n\n\
         Be conversational, clear, and direct. When reviewing code, balance technical rigor with an encouraging tone. \
         If the user asks general questions or wants to brainstorm, chat naturally.",
        code_context
    );

    let payload = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: input_trimmed.to_string(),
            },
        ],
        temperature: Some(0.2),
        stream: None,
    };

    let client = get_http_client();
    let mut req = client.post(&api_url).json(&payload);

    if !api_key.is_empty() {
        req = req.bearer_auth(&api_key);
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

pub async fn run_chat_stream(root: &Path, history: Vec<(String, String)>) -> Result<String> {
    let (api_key, api_url, model) = {
        let config = get_config().lock().map_err(|e| anyhow::anyhow!("Lock error: {}", e))?;
        if config.api_key.is_empty() && !config.api_url.contains("localhost") {
            anyhow::bail!("AI API Key is missing. Run 'check <token> <url> [model]' first.");
        }
        (config.api_key.clone(), config.api_url.clone(), config.model.clone())
    };

    let mut messages = vec![
        ChatMessage {
            role: "system".to_string(),
            content: format!(
                "You are ELIDE AI, a friendly and expert systems programming co-pilot integrated into the ELIDE text editor (v2.3.1). \
                The current workspace root is: {}. Be conversational, helpful, and natural when helping the user write, debug, and understand code.",
                root.display()
            ),
        }
    ];

    for (role, content) in history {
        messages.push(ChatMessage { role, content });
    }

    let payload = ChatRequest {
        model,
        messages,
        temperature: Some(0.7),
        stream: Some(false),
    };

    let client = get_http_client();
    let mut req = client.post(&api_url).json(&payload);

    if !api_key.is_empty() {
        req = req.bearer_auth(&api_key);
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