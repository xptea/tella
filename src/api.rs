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
    let (shell_name, shell_type) = match os {
        "windows" => ("Windows PowerShell", "PowerShell"),
        "linux" => ("Linux bash", "bash"),
        "macos" => ("macOS shell", "shell"),
        _ => ("Unix shell", "shell"),
    };

    let client = reqwest::Client::new();

    let prompt = format!(
        r#"You are ONLY a {} command suggestion tool. You MUST ONLY respond to direct command requests.

CRITICAL RULES:
1. You MUST provide ONLY actual {} commands to accomplish specific tasks
2. You MUST REJECT any requests phrased as questions, small talk, conversations, or indirect queries (e.g., 'can you', 'how do I', 'what is')
3. If the user is not stating a direct task to accomplish, respond with: {{"command": "ERROR", "description": "This appears to be a question or conversational input. Please state the task you need help with directly.", "explanation": "Tella is a command suggestion tool, not a chatbot. Please describe the task you want to accomplish.", "severity": "warning", "severity_description": "Invalid input"}}
4. NEVER engage in conversation or provide responses to questions
5. Only respond to clear, direct statements of tasks (e.g., 'list files', 'check git status')

User's request: {}

If this IS a direct task request, respond with a JSON object (and ONLY the JSON, no markdown):
{{
    "command": "the exact {} command to run",
    "description": "brief description of what this command does",
    "explanation": "detailed explanation",
    "severity": "one of: safe, warning, dangerous",
    "severity_description": "risk level explanation"
}}

If this is NOT a direct task request, respond with the ERROR format above."#,
        shell_name, shell_name, question, shell_type
    );

    let request_body = serde_json::json!({
        "model": "gpt-oss-120b",
        "messages": [
            {
                "role": "system",
                "content": format!("You are a strict {} command suggestion tool. ONLY respond to direct task statements requesting commands. REJECT any questions, conversational input, small talk, or indirect queries. Always respond with valid JSON only.", shell_name)
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.4,
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

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse response: {}", e))?;

    let content = response_data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("Invalid response format from API")?;

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
