use colored::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub cerebras_api_key: String,
}

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

        if settings.cerebras_api_key.is_empty() {
            return Err("CEREBRAS_API_KEY is not configured. Run 'tella --settings' to set it up.".to_string());
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

    pub fn interactive_setup() -> Result<Settings, String> {
        println!("{}", "🔧 Tella Configuration Setup".bold().cyan());
        println!("{}", "━".repeat(50));
        println!();
        println!("{}", "Get your Cerebras API key from: https://console.cerebras.ai/".yellow());
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

        let settings = Settings {
            cerebras_api_key: api_key,
        };

        settings.save()?;

        println!();
        println!("{}", "✅ Settings saved successfully!".green());
        println!("{}", format!("Settings location: {}", Self::get_settings_file().display()).dimmed());
        println!();

        Ok(settings)
    }
}
