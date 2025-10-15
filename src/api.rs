use serde::{Deserialize, Serialize};
use crate::settings::Settings;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    pub description: String,
    pub explanation: String,
    pub severity: String,
    pub severity_description: String,
}

pub async fn get_command_suggestion(question: &str) -> Result<CommandSuggestion, String> {
    // Get API key from settings
    let settings = Settings::load()?;
    let api_key = &settings.cerebras_api_key;

    let client = reqwest::Client::new();

    let prompt = format!(
        r#"You are ONLY a Windows PowerShell command suggestion tool. You MUST ONLY respond to legitimate command requests.

CRITICAL RULES:
1. You MUST provide ONLY actual Windows PowerShell commands to accomplish specific tasks
2. You MUST REJECT any non-command requests (small talk, questions, conversations, etc.)
3. If the user is not asking for a command, respond with: {{"command": "ERROR", "description": "This is not a command request. Please ask about a specific task you need help with.", "explanation": "Tella is a command suggestion tool, not a chatbot. Please ask what command you need to run.", "severity": "warning", "severity_description": "Invalid input"}}
4. NEVER engage in conversation or provide non-command responses
5. If unclear, ask for clarification about what task they want to accomplish

User's request: {}

If this IS a legitimate command request, respond with a JSON object (and ONLY the JSON, no markdown):
{{
    "command": "the exact PowerShell command to run",
    "description": "brief description of what this command does",
    "explanation": "detailed explanation",
    "severity": "one of: safe, warning, dangerous",
    "severity_description": "risk level explanation"
}}

If this is NOT a command request, respond with the ERROR format above."#,
        question
    );

    let request_body = serde_json::json!({
        "model": "gpt-oss-120b",
        "messages": [
            {
                "role": "system",
                "content": "You are a strict Windows PowerShell command suggestion tool. ONLY respond to command requests. REJECT any conversational input, small talk, or non-command questions. Always respond with valid JSON only."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.7,
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

    // Extract the assistant's message
    let content = response_data
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .ok_or("Invalid response format from API")?;

    // Try to parse the JSON from the content
    let parsed: CommandSuggestion = {
        let mut result = serde_json::from_str(content);
        
        if result.is_err() {
            // Try to extract JSON if it's embedded in markdown code blocks
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
