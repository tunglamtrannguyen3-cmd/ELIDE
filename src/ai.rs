// src/ai.rs
use anyhow::Result;
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Component, Path};
use std::sync::{Mutex, OnceLock};
use std::time::Duration;

// ==========================================
// 1. CONFIGURATION STRUCT & STATE MANAGEMENT
// ==========================================

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

// ==========================================
// 2. DATA STRUCTURES
// ==========================================

#[derive(Serialize, Clone, Debug)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

impl<'de> Deserialize<'de> for ChatMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct RawChatMessage {
            role: String,
            #[serde(default)]
            content: Option<serde_json::Value>,
        }

        let raw = RawChatMessage::deserialize(deserializer)?;
        let content_str = match raw.content {
            Some(serde_json::Value::String(s)) => s,
            Some(serde_json::Value::Null) | None => String::new(),
            Some(other) => other.to_string(),
        };

        Ok(ChatMessage {
            role: raw.role,
            content: content_str,
        })
    }
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

// ==========================================
// 3. COMMAND PARSING & CONFIGURATION HELPERS
// ==========================================

pub(crate) fn try_handle_command(input_trimmed: &str) -> Result<Option<String>, String> {
    // 1. Connection Command
    let command_payload = input_trimmed
        .strip_prefix("connect:")
        .or_else(|| input_trimmed.strip_prefix("connect "));

    if let Some(rest) = command_payload {
        let parts: Vec<&str> = rest.split_whitespace().collect();
        if parts.len() >= 2 {
            let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
            config.api_key = parts[0].to_string();
            config.api_url = parts[1].to_string();

            if parts.len() >= 3 {
                config.model = parts[2].to_string();
            }

            return Ok(Some(format!(
                "🔑 Connection saved successfully!\nEndpoint: {}\nModel: {}\n\nYou can now run prompts to audit your workspace.",
                config.api_url, config.model
            )));
        } else {
            return Err("Usage for connection: connect: <token> <url> [model] or connect <token> <url> [model]".to_string());
        }
    }

    // 2. System Prompt Alias Parsing (supports `set-sysprompt` and `prompt:`)
    let prompt_payload = input_trimmed
        .strip_prefix("set-sysprompt ")
        .or_else(|| input_trimmed.strip_prefix("set-sysprompt:"))
        .or_else(|| input_trimmed.strip_prefix("prompt:"));

    if let Some(new_prompt) = prompt_payload {
        let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
        config.system_prompt = new_prompt.trim().to_string();
        return Ok(Some(format!(
            "✨ System prompt updated successfully!\nNew Prompt:\n{}",
            config.system_prompt
        )));
    }

    // 3. Max Context Character Limit Alias Parsing (supports `set-max-lim` and `limit:`)
    let limit_payload = input_trimmed
        .strip_prefix("set-max-lim ")
        .or_else(|| input_trimmed.strip_prefix("set-max-lim:"))
        .or_else(|| input_trimmed.strip_prefix("limit:"));

    if let Some(limit_str) = limit_payload {
        if let Ok(new_limit) = limit_str.trim().parse::<usize>() {
            let mut config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
            config.max_context_chars = new_limit;
            return Ok(Some(format!(
                "📊 Workspace context limit updated successfully!\nNew Character Limit: {} chars",
                config.max_context_chars
            )));
        } else {
            return Err("Usage for limit: set-max-lim <number> or limit: <number> (e.g. set-max-lim 150000)".to_string());
        }
    }

    Ok(None)
}

// ==========================================
// 4. WORKSPACE SCANNING & CONTEXT GATHERING
// ==========================================

fn gather_workspace_context(workspace_root: &Path, max_chars: usize) -> String {
    let mut code_context = String::new();
    let walker = WalkBuilder::new(workspace_root)
        .hidden(true)
        .git_ignore(true)
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if path.is_file() {
            let is_excluded_dir = path.components().any(|comp| {
                if let Component::Normal(name) = comp {
                    let name_str = name.to_string_lossy().to_lowercase();
                    matches!(name_str.as_str(), "target" | "node_modules" | ".git" | "build" | "dist")
                } else {
                    false
                }
            });

            if is_excluded_dir {
                continue;
            }

            let file_name = path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_lowercase();

            if file_name.starts_with(".env")
                || file_name.ends_with(".pem")
                || file_name.ends_with(".key")
                || file_name.ends_with(".cert")
                || file_name.ends_with(".pfx")
                || file_name.ends_with(".p12")
                || file_name.ends_with(".log")
                || file_name == "credentials.json"
                || file_name == "secrets.json"
                || file_name == ".ds_store"
            {
                continue;
            }

            if let Ok(content) = fs::read_to_string(path) {
                let rel_path = path
                    .strip_prefix(workspace_root)
                    .unwrap_or(path)
                    .display();
                let file_snippet = format!(
                    "\n--- FILE: {} ---\n{}\n",
                    rel_path, content
                );

                if code_context.chars().count() + file_snippet.chars().count() > max_chars {
                    code_context.push_str("\n[Context truncated due to character limits...]\n");
                    break;
                }

                code_context.push_str(&file_snippet);
            }
        }
    }
    code_context
}

// ==========================================
// 5. MAIN CHAT STREAM & EXECUTION
// ==========================================

pub async fn run_chat_stream(
    workspace_root: &Path,
    messages: Vec<(String, String)>,
) -> Result<String, String> {
    if let Some((role, content)) = messages.last() {
        let input_trimmed = content.trim();
        if role == "user" {
            if let Some(response) = try_handle_command(input_trimmed)? {
                return Ok(response);
            }
        }
    }

    let (api_key, api_url, model, system_prompt, max_context_chars) = {
        let config = get_config().lock().map_err(|e| format!("Lock error: {}", e))?;
        if config.api_key.is_empty() && !config.api_url.contains("localhost") {
            return Err(
                "❌ AI Not Connected!\nPlease configure your credentials first using:\nconnect: <token> <url> [model]\n\nCommands:\nset-sysprompt <prompt>\nset-max-lim <number>"
                    .to_string(),
            );
        }
        (
            config.api_key.clone(),
            config.api_url.clone(),
            config.model.clone(),
            config.system_prompt.clone(),
            config.max_context_chars,
        )
    };

    let code_context = gather_workspace_context(workspace_root, max_context_chars);

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

pub async fn run_check(workspace_root: &Path, input: &str) -> Result<String, String> {
    run_chat_stream(workspace_root, vec![("user".to_string(), input.to_string())]).await
}

// ==========================================
// 6. UNIT TESTS
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_handle_command_connect() {
        let result = try_handle_command("connect: my_token https://api.openai.com/v1/chat/completions gpt-4");
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("Connection saved successfully!"));
    }

    #[test]
    fn test_try_handle_command_connect_short() {
        let result = try_handle_command("connect my_token https://api.openai.com/v1/chat/completions");
        assert!(result.is_ok());
        assert!(result.unwrap().is_some());
    }

    #[test]
    fn test_try_handle_command_connect_invalid() {
        let result = try_handle_command("connect: invalid_only");
        assert!(result.is_err());
    }

    #[test]
    fn test_try_handle_command_sysprompt() {
        let result = try_handle_command("set-sysprompt Be a helpful assistant.");
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("System prompt updated successfully!"));

        let result2 = try_handle_command("prompt: Another prompt");
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_some());
    }

    #[test]
    fn test_try_handle_command_max_lim() {
        let result = try_handle_command("set-max-lim 100000");
        assert!(result.is_ok());
        let msg = result.unwrap();
        assert!(msg.is_some());
        assert!(msg.unwrap().contains("Workspace context limit updated successfully!"));

        let result2 = try_handle_command("limit: 50000");
        assert!(result2.is_ok());
        assert!(result2.unwrap().is_some());

        let result_invalid = try_handle_command("set-max-lim not_a_number");
        assert!(result_invalid.is_err());
    }

    #[test]
    fn test_try_handle_command_unknown() {
        let result = try_handle_command("random_command_xyz");
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn test_chat_message_deserialization() {
        let json_str = r#"{"role": "user", "content": "Hello world"}"#;
        let msg: ChatMessage = serde_json::from_str(json_str).unwrap();
        assert_eq!(msg.role, "user");
        assert_eq!(msg.content, "Hello world");

        let json_null = r#"{"role": "assistant", "content": null}"#;
        let msg_null: ChatMessage = serde_json::from_str(json_null).unwrap();
        assert_eq!(msg_null.role, "assistant");
        assert_eq!(msg_null.content, "");

        let json_number = r#"{"role": "system", "content": 12345}"#;
        let msg_num: ChatMessage = serde_json::from_str(json_number).unwrap();
        assert_eq!(msg_num.role, "system");
        assert_eq!(msg_num.content, "12345");
    }
    #[test]
    fn test_gather_workspace_context() {
        use std::fs::File;
        use std::io::Write;

        let temp_dir = tempfile::tempdir().unwrap();
        let root = temp_dir.path();

        // Create a valid source file
        let src_file_path = root.join("main.rs");
        let mut f1 = File::create(&src_file_path).unwrap();
        writeln!(f1, "fn main() {{ println!(\"Hello\"); }}").unwrap();

        // Create an excluded directory containing a file that should be skipped
        let target_dir = root.join("target");
        fs::create_dir(&target_dir).unwrap();
        let ignored_file_path = target_dir.join("build.rs");
        let mut f2 = File::create(&ignored_file_path).unwrap();
        writeln!(f2, "should not be included").unwrap();

        // Create a sensitive file that should be filtered out
        let env_file_path = root.join(".env");
        let mut f3 = File::create(&env_file_path).unwrap();
        writeln!(f3, "SECRET_KEY=12345").unwrap();

        let context = gather_workspace_context(root, 10_000);
        
        assert!(context.contains("main.rs"), "Context should include main.rs");
        assert!(!context.contains("should not be included"), "Context must exclude files in target/ directory");
        assert!(!context.contains("SECRET_KEY"), "Context must filter out sensitive .env files");
    }
}