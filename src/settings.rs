use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputSettings {
    pub show_command: bool,
    pub show_description: bool,
    pub show_explanation: bool,
    pub show_severity: bool,
}

impl Default for OutputSettings {
    fn default() -> Self {
        OutputSettings {
            show_command: true,
            show_description: true,
            show_explanation: true,
            show_severity: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub provider: String, // "ollama" or "cerebras"
    pub cerebras_api_key: Option<String>,
    pub ollama_model: Option<String>,
    pub ollama_base_url: Option<String>,
    #[serde(default)]
    pub output_settings: OutputSettings,
}

pub const CEREBRAS_MODELS: &[&str] = &[
    "llama3.3-70b",
    "llama3.1-8b",
    "gpt-oss-120b",
    "qwen-3-235b-a22b-instruct-2507",
    "qwen-3-235b-a22b-thinking-2507",
    "qwen-3-coder-480b",
];

impl Settings {
    pub fn get_settings_path() -> PathBuf {
        let app_data = if cfg!(target_os = "windows") {
            dirs::data_dir().unwrap_or_else(|| PathBuf::from("."))
        } else {
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("."))
        };

        let tella_dir = app_data.join("tella");
        tella_dir
    }

    pub fn get_settings_file() -> PathBuf {
        Self::get_settings_path().join("settings.json")
    }

    pub fn load() -> Result<Settings, String> {
        let settings_file = Self::get_settings_file();

        if !settings_file.exists() {
            return Err("Settings file not found. Run 'tella --settings' to configure.".to_string());
        }

        let content = fs::read_to_string(&settings_file)
            .map_err(|e| format!("Failed to read settings file: {}", e))?;

        let settings: Settings = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse settings file: {}", e))?;

        // Validate based on provider
        match settings.provider.as_str() {
            "cerebras" => {
                if settings.cerebras_api_key.is_none() || settings.cerebras_api_key.as_ref().map_or(true, |k| k.is_empty()) {
                    return Err("CEREBRAS_API_KEY is not configured. Run 'tella --settings' to set it up.".to_string());
                }
            }
            "ollama" => {
                if settings.ollama_model.is_none() || settings.ollama_model.as_ref().map_or(true, |m| m.is_empty()) {
                    return Err("Ollama model is not configured. Run 'tella --settings' to set it up.".to_string());
                }
            }
            _ => return Err("Invalid provider in settings. Must be 'ollama' or 'cerebras'.".to_string()),
        }

        Ok(settings)
    }

    pub fn save(&self) -> Result<(), String> {
        let settings_dir = Self::get_settings_path();
        let settings_file = Self::get_settings_file();

        fs::create_dir_all(&settings_dir)
            .map_err(|e| format!("Failed to create settings directory: {}", e))?;

        let content = serde_json::to_string_pretty(&self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&settings_file, content)
            .map_err(|e| format!("Failed to write settings file: {}", e))?;

        Ok(())
    }

    pub async fn interactive_setup() -> Result<Settings, String> {
        println!("{}", "🔧 Tella Configuration Setup".bold().cyan());
        println!("{}", "━".repeat(50));
        println!();

        // Provider selection
        println!("{}", "Which model provider would you like to use?".bold());
        println!();
        println!("  {} Ollama (Local, fully offline, free)", "1.".cyan());
        println!("  {} Cerebras (Cloud-based, requires API key)", "2.".cyan());
        println!();

        print!("{} ", "Choose (1 or 2):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;

        let mut choice = String::new();
        io::stdin()
            .read_line(&mut choice)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let choice = choice.trim();

        let settings = match choice {
            "1" => Self::setup_ollama().await?,
            "2" => Self::setup_cerebras()?,
            _ => return Err("Invalid choice. Please enter 1 or 2.".to_string()),
        };

        settings.save()?;

        println!();
        println!("{}", "✅ Settings saved successfully!".green());
        println!("{}", format!("Settings location: {}", Self::get_settings_file().display()).dimmed());
        println!();

        Ok(settings)
    }

    async fn setup_ollama() -> Result<Settings, String> {
        println!();
        println!("{}", "🎯 Ollama Setup".bold().cyan());
        println!("{}", "━".repeat(50));
        println!();
        println!("{}", "Make sure Ollama is installed and running.".yellow());
        println!("{}", "Default URL: http://localhost:11434".dimmed());
        println!();

        // Get base URL
        print!("{} ", "Enter Ollama base URL (press Enter for default):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;

        let mut base_url = String::new();
        io::stdin()
            .read_line(&mut base_url)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let base_url = match base_url.trim() {
            "" => "http://localhost:11434".to_string(),
            url => url.to_string(),
        };

        // Try to fetch available models
        println!();
        println!("{}", "Fetching available Ollama models...".cyan());

        let available_models = match Self::fetch_ollama_models(&base_url).await {
            Ok(models) => {
                println!("{}", format!("✅ Found {} models", models.len()).green());
                models
            }
            Err(e) => {
                eprintln!("{}", format!("⚠️  Could not fetch models: {}", e).yellow());
                eprintln!("{}", "You may need to start Ollama first.".yellow());
                vec![]
            }
        };

        println!();
        if available_models.is_empty() {
            println!("{}", "No models found. Available commands:".yellow());
            println!("  {} ollama list", "$".cyan());
            println!("  {} ollama pull llama2 (or another model)", "$".cyan());
            println!();
            print!("{} ", "Enter Ollama model name manually:".bold());
        } else {
            println!("{}", "Available models:".bold());
            for (i, model) in available_models.iter().enumerate() {
                println!("  {}) {}", i + 1, model);
            }
            println!();
            print!("{} ", "Select model number or enter custom name:".bold());
        }

        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;

        let mut model_choice = String::new();
        io::stdin()
            .read_line(&mut model_choice)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let model_choice = model_choice.trim();

        let ollama_model = if let Ok(idx) = model_choice.parse::<usize>() {
            if idx > 0 && idx <= available_models.len() {
                available_models[idx - 1].clone()
            } else {
                return Err("Invalid selection.".to_string());
            }
        } else {
            model_choice.to_string()
        };

        if ollama_model.is_empty() {
            return Err("Model name cannot be empty".to_string());
        }

        Ok(Settings {
            provider: "ollama".to_string(),
            ollama_model: Some(ollama_model),
            ollama_base_url: Some(base_url),
            cerebras_api_key: None,
            output_settings: Self::setup_output_settings()?,
        })
    }

    fn setup_cerebras() -> Result<Settings, String> {
        println!();
        println!("{}", "🎯 Cerebras Setup".bold().cyan());
        println!("{}", "━".repeat(50));
        println!();
        println!("{}", "Get your API key from: https://console.cerebras.ai/".yellow());
        println!();
        println!("{}", "Available models:".bold());
        for model in CEREBRAS_MODELS {
            println!("  • {}", model);
        }
        println!();

        print!("{} ", "Enter your Cerebras API key:".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;

        let mut api_key = String::new();
        io::stdin()
            .read_line(&mut api_key)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let api_key = api_key.trim().to_string();

        if api_key.is_empty() {
            return Err("API key cannot be empty".to_string());
        }

        // Ask which model to use
        println!();
        println!("{}", "Which Cerebras model would you like to use?".bold());
        for (i, model) in CEREBRAS_MODELS.iter().enumerate() {
            println!("  {}) {}", i + 1, model);
        }
        println!();
        print!("{} ", "Select model number:".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;

        let mut model_choice = String::new();
        io::stdin()
            .read_line(&mut model_choice)
            .map_err(|e| format!("Failed to read input: {}", e))?;

        let model_idx = model_choice
            .trim()
            .parse::<usize>()
            .map_err(|_| "Invalid selection.".to_string())?;

        if model_idx == 0 || model_idx > CEREBRAS_MODELS.len() {
            return Err("Invalid selection.".to_string());
        }

        Ok(Settings {
            provider: "cerebras".to_string(),
            cerebras_api_key: Some(api_key),
            ollama_model: Some(CEREBRAS_MODELS[model_idx - 1].to_string()),
            ollama_base_url: None,
            output_settings: Self::setup_output_settings()?,
        })
    }

    fn setup_output_settings() -> Result<OutputSettings, String> {
        println!();
        println!("{}", "📋 Output Settings".bold().cyan());
        println!("{}", "━".repeat(50));
        println!();
        println!("{}", "Choose which fields you want to display:".bold());
        println!();

        let mut settings = OutputSettings::default();

        // Command
        print!("{} ", "Show command? (Y/n):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;
        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;
        settings.show_command = !input.trim().eq_ignore_ascii_case("n");

        // Description
        print!("{} ", "Show description? (Y/n):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;
        settings.show_description = !input.trim().eq_ignore_ascii_case("n");

        // Explanation
        print!("{} ", "Show explanation? (Y/n):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;
        settings.show_explanation = !input.trim().eq_ignore_ascii_case("n");

        // Severity
        print!("{} ", "Show severity? (Y/n):".bold());
        io::stdout().flush().map_err(|e| format!("IO error: {}", e))?;
        input.clear();
        io::stdin()
            .read_line(&mut input)
            .map_err(|e| format!("Failed to read input: {}", e))?;
        settings.show_severity = !input.trim().eq_ignore_ascii_case("n");

        println!();
        Ok(settings)
    }

    async fn fetch_ollama_models(base_url: &str) -> Result<Vec<String>, String> {
        let url = format!("{}/api/tags", base_url);
        let client = reqwest::Client::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            client.get(&url).send(),
        )
        .await
        {
            Ok(Ok(response)) => {
                let body = response
                    .text()
                    .await
                    .map_err(|e| format!("Failed to read response: {}", e))?;

                let json: serde_json::Value = serde_json::from_str(&body)
                    .map_err(|e| format!("Failed to parse response: {}", e))?;

                let models = json
                    .get("models")
                    .and_then(|m| m.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|m| m.get("name").and_then(|n| n.as_str()).map(|s| s.to_string()))
                            .collect()
                    })
                    .unwrap_or_default();

                Ok(models)
            }
            Ok(Err(e)) => Err(format!("Connection failed: {}", e)),
            Err(_) => Err("Connection timeout".to_string()),
        }
    }
}
