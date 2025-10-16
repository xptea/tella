use serde::{Deserialize, Serialize};
use crate::settings::Settings;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub explanation: String,
    pub severity: String,
    pub severity_description: String,
}

pub async fn get_command_suggestion(question: &str) -> Result<CommandSuggestion, String> {
    let settings = Settings::load()?;
    let api_key = &settings.cerebras_api_key;

    let os = env::consts::OS;
    let shell_type = match os {
        "windows" => "PowerShell",
        "linux" => "bash",
        "macos" => "shell",
        _ => "shell",
    };

    let client = reqwest::Client::new();

    let prompt = format!(
        r#"Suggest the best {} command for: {}

If it's a task, respond with JSON:
{{
    "command": "exact command",
    "description": "brief desc",
    "explanation": "details",
    "severity": "safe|warning|dangerous",
    "severity_description": "risk"
}}

If not a task, use "no command returned"."#,
        shell_type, question
    );

    let request_body = serde_json::json!({
        "model": "llama3.3-70b",
        "messages": [
            {
                "role": "system",
                "content": "You are a command suggestion tool. Suggest commands or 'no command returned'. Always JSON."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.3,
        "max_tokens": 500
    });

    let response = client
        .post("https://api.cerebras.ai/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // eprintln!("🔍 Debug: Full API response: {}", response_text);

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    // Check for API error responses
    if let Some(error_type) = response_data.get("type").and_then(|t| t.as_str()) {
        if error_type == "too_many_requests_error" {
            return Err(response_data.get("message").and_then(|m| m.as_str()).unwrap_or("API rate limit exceeded").to_string());
        }
        // Add other error types if needed
    }

    let content = response_data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("Invalid response format from API")?;

    // eprintln!("🔍 Debug: API raw response: {}", content);

    let parsed: CommandSuggestion = {
        let mut result = serde_json::from_str(content);
        
        if result.is_err() {
            if let Some(start) = content.find('{') {
                if let Some(end) = content.rfind('}') {
                    if end > start {
                        result = serde_json::from_str(&content[start..=end]);
                    }
                }
            }
        }
        
        result.map_err(|e| format!("Failed to parse command suggestion: {}", e))?
    };

    Ok(parsed)
}
