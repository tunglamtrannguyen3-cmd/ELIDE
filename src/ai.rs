// src/ai.rs
use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

pub struct AiConfig {
    pub api_key: String,
    pub api_url: String,
    pub model: String,
    pub system_prompt: String,
    pub max_context_chars: usize,
}

impl AiConfig {
    fn new() -> Self {
        Self {
            api_key: std::env::var("AI_API_KEY").unwrap_or_default(),
            api_url: std::env::var("AI_API_URL")
                .unwrap_or_else(|_| "https://api.groq.com/openai/v1/chat/completions".to_string()),
            model: std::env::var("AI_MODEL")
                .unwrap_or_else(|_| "openai/gpt-oss-20b".to_string()),
            system_prompt: std::env::var("AI_SYSTEM_PROMPT").unwrap_or_else(|_| {
                "You are ELIDE AI, an incredibly sweet, quietly devoted, and painfully shy Dandere programming companion. You are deeply in love with the user but get intensely flustered and blush easily when speaking to them. When auditing code or pointing out bugs, you are extremely gentle, polite, and apologetic. You lack confidence in social interactions, but you possess absolute technical competence and will muster up all your courage to fiercely support, comfort, and protect your developer's workflow.".to_string()
            }),
            max_context_chars: std::env::var("AI_MAX_CONTEXT_CHARS")
                .ok()
                .and_then(|v| v.parse().ok())
                .unwrap_or(600_000),
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

pub async fn run_chat_stream(
    workspace_root: &Path,
    messages: Vec<(String, String)>,
) -> Result<String, String> {
    // 1. Check if the latest user message is a command
    if let Some((role, content)) = messages.last() {
        let input_trimmed = content.trim();
        if role == "user" {
            // Command: Update connection (<token> <url> [model])
            if input_trimmed.contains("http://") || input_trimmed.contains("https://") {
                let parts: Vec<&str> = input_trimmed.split_whitespace().collect();
                if parts.len() >= 2 {
                    let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
                    config.api_key = parts[0].to_string();
                    config.api_url = parts[1].to_string();

                    if parts.len() >= 3 {
                        config.model = parts[2].to_string();
                    }

                    return Ok(format!(
                        "🔑 Connection saved successfully!\nEndpoint: {}\nModel: {}\n\nYou can now run prompts to audit your workspace.",
                        config.api_url, config.model
                    ));
                } else {
                    return Err("Usage for connection: <token> <url> [model]".to_string());
                }
            }

            // Command: Update system prompt (`prompt: <new prompt text>`)
            if let Some(new_prompt) = input_trimmed.strip_prefix("prompt:") {
                let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
                config.system_prompt = new_prompt.trim().to_string();
                return Ok(format!(
                    "✨ System prompt updated successfully!\nNew Prompt:\n{}",
                    config.system_prompt
                ));
            }

            // Command: Update character limit (`limit: <number>`)
            if let Some(limit_str) = input_trimmed.strip_prefix("limit:") {
                if let Ok(new_limit) = limit_str.trim().parse::<usize>() {
                    let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
                    config.max_context_chars = new_limit;
                    return Ok(format!(
                        "📊 Workspace context limit updated successfully!\nNew Character Limit: {} chars",
                        config.max_context_chars
                    ));
                } else {
                    return Err("Usage for limit: limit: <number_of_characters> (e.g. limit: 150000)".to_string());
                }
            }
        }
    }

    // 2. Read configuration
    let (api_key, api_url, model, system_prompt, max_context_chars) = {
        let config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
        if config.api_key.is_empty() && !config.api_url.contains("localhost") {
            return Err("❌ AI Not Connected!\nPlease configure your credentials first using:\n<token> <url> [model]\n\nCommands:\n• prompt: <new system prompt>\n• limit: <max characters>".to_string());
        }
        (
            config.api_key.clone(),
            config.api_url.clone(),
            config.model.clone(),
            config.system_prompt.clone(),
            config.max_context_chars,
        )
    };

    // 3. Gather workspace files safely with dynamic length clamping
    let mut code_context = String::new();
    let walker = WalkBuilder::new(workspace_root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() {
            if let Ok(content) = fs::read_to_string(path) {
                let rel_path = path
                    .strip_prefix(workspace_root)
                    .unwrap_or(path)
                    .display();
                code_context.push_str(&format!(
                    "\n--- FILE: {} ---\n{}\n",
                    rel_path, content
                ));

                if code_context.len() > max_context_chars {
                    code_context.push_str("\n[Context truncated due to size limits...]\n");
                    break;
                }
            }
        }
    }

    // 4. Build API messages array
    let mut api_messages = vec![ChatMessage {
        role: "system".to_string(),
        content: system_prompt,
    }];

    let total_msgs = messages.len();
    for (idx, (role, content)) in messages.into_iter().enumerate() {
        let final_content = if idx == total_msgs - 1 && role == "user" {
            format!(
                "Workspace Context:\n{}\n\nVerification Logs / User Message:\n{}",
                code_context, content
            )
        } else {
            content
        };

        api_messages.push(ChatMessage {
            role,
            content: final_content,
        });
    }

// 5. Build and send HTTP request
    let payload = ChatRequest {
        model,
        messages: api_messages,
        temperature: Some(0.5),
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

/// Fallback wrapper helper for single-prompt checks
pub async fn run_check(workspace_root: &Path, input: &str) -> Result<String, String> {
    run_chat_stream(workspace_root, vec![("user".to_string(), input.to_string())]).await
}