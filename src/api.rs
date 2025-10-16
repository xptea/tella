use serde::{Deserialize, Serialize};
use crate::settings::Settings;
use std::env;
use colored::*;

// Set to true to enable debug output, false to disable
const DEBUG: bool = false;

macro_rules! debug_print {
    ($($arg:tt)*) => {
        if DEBUG {
            eprintln!($($arg)*)
        }
    };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandSuggestion {
    pub command: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub explanation: String,
    #[serde(default)]
    pub severity: String,
    #[serde(default)]
    pub severity_description: String,
}

pub async fn get_command_suggestion(question: &str) -> Result<CommandSuggestion, String> {
    let settings = Settings::load()?;

    match settings.provider.as_str() {
        "ollama" => get_command_from_ollama(question, &settings).await,
        "cerebras" => get_command_from_cerebras(question, &settings).await,
        _ => Err("Invalid provider in settings".to_string()),
    }
}

async fn get_command_from_ollama(question: &str, settings: &Settings) -> Result<CommandSuggestion, String> {
    let base_url = settings
        .ollama_base_url
        .as_ref()
        .map(|s| s.as_str())
        .unwrap_or("http://localhost:11434");

    let model = settings
        .ollama_model
        .as_ref()
        .ok_or("Ollama model not configured")?;

    let os = env::consts::OS;
    let shell_type = match os {
        "windows" => "PowerShell",
        "linux" => "bash",
        "macos" => "shell",
        _ => "shell",
    };

    let client = reqwest::Client::new();
    let url = format!("{}/api/generate", base_url);

    // Build the JSON response format based on output settings
    let mut json_fields = vec![];
    if settings.output_settings.show_command {
        json_fields.push("\"command\": \"exact command\"");
    }
    if settings.output_settings.show_description {
        json_fields.push("\"description\": \"brief desc\"");
    }
    if settings.output_settings.show_severity {
        json_fields.push("\"severity\": \"safe|warning|dangerous\"");
        json_fields.push("\"severity_description\": \"risk\"");
    }
    
    let json_format = json_fields.join(",\n    ");

    // First call: Get command and description only
    let prompt = format!(
        r#"Suggest the best {} command for: {}

Respond with ONLY valid JSON (no markdown, no extra text):
{{
    {}
}}

If not a task, use "no command returned" for command."#,
        shell_type, question, json_format
    );

    let request_body = serde_json::json!({
        "model": model,
        "prompt": prompt,
        "temperature": 0.3,
        "stream": false,
        "keep_alive": "5m"
    });

    debug_print!("🔍 [OLLAMA DEBUG - FIRST REQUEST]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("URL: {}", url);
    debug_print!("Model: {}", model);
    debug_print!("Base URL: {}", base_url);
    debug_print!("Timeout: 120 seconds");
    debug_print!("Output Settings:");
    debug_print!("  show_command: {}", settings.output_settings.show_command);
    debug_print!("  show_description: {}", settings.output_settings.show_description);
    debug_print!("  show_severity: {}", settings.output_settings.show_severity);
    debug_print!("  show_explanation: {}", settings.output_settings.show_explanation);
    debug_print!("Request Body:");
    debug_print!("{}", serde_json::to_string_pretty(&request_body).unwrap_or_default());
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    let response = match tokio::time::timeout(
        std::time::Duration::from_secs(120),
        client.post(&url).json(&request_body).send(),
    )
    .await
    {
        Ok(Ok(resp)) => resp,
        Ok(Err(e)) => {
            eprintln!("{} {}", "❌ Connection Error:".red().bold(), e);
            return Err(format!("❌ Ollama connection failed: {}. Make sure Ollama is running on {}", e, base_url));
        }
        Err(_) => {
            debug_print!("❌ Request Timeout (120 seconds exceeded)");
            debug_print!("This usually means:");
            debug_print!("  • Ollama is still loading the model (first run)");
            debug_print!("  • The model is too large for your system");
            debug_print!("  • Check Ollama logs for errors");
            return Err(format!("❌ Ollama request timeout after 120 seconds on {}. Is the model too large or is Ollama still loading?", base_url));
        }
    };

    let response_text = response
        .text()
        .await
        .map_err(|e| format!("Failed to read Ollama response: {}", e))?;

    debug_print!("🔍 [OLLAMA DEBUG - FIRST RESPONSE]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("Raw Response Text:");
    debug_print!("{}", response_text);
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    let response_data: serde_json::Value = serde_json::from_str(&response_text)
        .map_err(|e| format!("Failed to parse Ollama response: {}", e))?;

    debug_print!("🔍 [OLLAMA DEBUG - PARSED JSON]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("{}", serde_json::to_string_pretty(&response_data).unwrap_or_default());
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    let content = response_data
        .get("response")
        .and_then(|c| c.as_str())
        .ok_or("Invalid response format from Ollama")?;

    debug_print!("🔍 [OLLAMA DEBUG - EXTRACTED CONTENT]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("{}", content);
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    let mut parsed: CommandSuggestion = {
        let mut result = serde_json::from_str(content);

        if result.is_err() {
            debug_print!("⚠️  First parse attempt failed, trying to extract JSON...");
            
            // Try to remove markdown code block formatting (```json ... ```)
            let mut clean_content = content.to_string();
            if clean_content.starts_with("```") {
                // Remove opening ```json or ```
                if let Some(start_idx) = clean_content.find('\n') {
                    clean_content = clean_content[start_idx + 1..].to_string();
                }
            }
            if clean_content.ends_with("```") {
                clean_content.truncate(clean_content.len() - 3);
            }
            
            // Now try to extract JSON
            if let Some(start) = clean_content.find('{') {
                if let Some(end) = clean_content.rfind('}') {
                    if end > start {
                        let extracted = &clean_content[start..=end];
                        debug_print!("Extracted JSON (after markdown cleanup):");
                        debug_print!("{}", extracted);
                        result = serde_json::from_str(extracted);
                    }
                }
            }
        }

        result.map_err(|e| format!("Failed to parse command suggestion: {}", e))?
    };

    debug_print!("🔍 [OLLAMA DEBUG - PARSED COMMAND SUGGESTION]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("{}", serde_json::to_string_pretty(&parsed).unwrap_or_default());
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    // Fill in missing fields with defaults based on output settings
    if !settings.output_settings.show_description && parsed.description.is_empty() {
        parsed.description = String::new();
    }
    if !settings.output_settings.show_severity {
        parsed.severity = String::new();
        parsed.severity_description = String::new();
    }

    // If command is "ERROR" or "no command returned", return early without explanation
    if parsed.command == "ERROR" || parsed.command == "no command returned" {
        debug_print!("ℹ️  No command returned, skipping explanation request");
        parsed.explanation = "Unable to find a suitable command for this request.".to_string();
        return Ok(parsed);
    }

    // Only fetch explanation if it's enabled in output settings
    if !settings.output_settings.show_explanation {
        debug_print!("ℹ️  Explanation disabled in settings, skipping");
        parsed.explanation = String::new();
        return Ok(parsed);
    }

    // Second call: Get explanation (async, separate)
    let explanation_prompt = format!(
        r#"Provide a detailed explanation for this {} command: {}

Respond with ONLY valid JSON (no markdown, no extra text):
{{
    "explanation": "detailed explanation of what this command does and why it's recommended"
}}
"#,
        shell_type, parsed.command
    );

    let explanation_body = serde_json::json!({
        "model": model,
        "prompt": explanation_prompt,
        "temperature": 0.3,
        "stream": false,
        "keep_alive": "5m"
    });

    debug_print!("{}", "🔍 [OLLAMA DEBUG - SECOND REQUEST (EXPLANATION)]".cyan().bold());
    debug_print!("{}", "─".repeat(60).cyan());
    debug_print!("{}", "Request Body:".cyan().bold());
    debug_print!("{}", serde_json::to_string_pretty(&explanation_body).unwrap_or_default());
    debug_print!("{}", "─".repeat(60).cyan());
    debug_print!();

    // Don't fail if explanation fetch fails, just use a default
    if let Ok(Ok(exp_response)) = tokio::time::timeout(
        std::time::Duration::from_secs(60),
        client.post(&url).json(&explanation_body).send(),
    )
    .await
    {
        if let Ok(exp_text) = exp_response.text().await {
            debug_print!("{}", "🔍 [OLLAMA DEBUG - SECOND RESPONSE]".cyan().bold());
            debug_print!("{}", "─".repeat(60).cyan());
            debug_print!("{}", "Raw Response Text:".cyan().bold());
            debug_print!("{}", exp_text);
            debug_print!("{}", "─".repeat(60).cyan());
            debug_print!();

            if let Ok(exp_data) = serde_json::from_str::<serde_json::Value>(&exp_text) {
                debug_print!("{}", "🔍 [OLLAMA DEBUG - EXPLANATION PARSED]".cyan().bold());
                debug_print!("{}", "─".repeat(60).cyan());
                debug_print!("{}", serde_json::to_string_pretty(&exp_data).unwrap_or_default());
                debug_print!("{}", "─".repeat(60).cyan());
                debug_print!();

                if let Some(exp_content) = exp_data.get("response").and_then(|c| c.as_str()) {
                    if let Ok(exp_json) = serde_json::from_str::<serde_json::Value>(exp_content) {
                        if let Some(explanation) = exp_json.get("explanation").and_then(|e| e.as_str()) {
                            debug_print!("{} {}", "✅ Explanation found:".green().bold(), explanation);
                            parsed.explanation = explanation.to_string();
                        }
                    } else if let Some(start) = exp_content.find('{') {
                        if let Some(end) = exp_content.rfind('}') {
                            if end > start {
                                if let Ok(exp_json) = serde_json::from_str::<serde_json::Value>(&exp_content[start..=end]) {
                                    if let Some(explanation) = exp_json.get("explanation").and_then(|e| e.as_str()) {
                                        debug_print!("{} {}", "✅ Explanation found (extracted):".green().bold(), explanation);
                                        parsed.explanation = explanation.to_string();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    } else {
        debug_print!("{}", "⚠️  Explanation request timed out or failed".yellow().bold());
    }

    debug_print!("🔍 [OLLAMA DEBUG - FINAL RESULT]");
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!("{}", serde_json::to_string_pretty(&parsed).unwrap_or_default());
    debug_print!("────────────────────────────────────────────────────────────");
    debug_print!();

    Ok(parsed)
}

async fn get_command_from_cerebras(question: &str, settings: &Settings) -> Result<CommandSuggestion, String> {
    let api_key = settings
        .cerebras_api_key
        .as_ref()
        .ok_or("Cerebras API key not configured")?;

    let model = settings
        .ollama_model
        .as_ref()
        .ok_or("Cerebras model not configured")?;

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
        "model": model,
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
