use serde::Deserialize;
use colored::*;

#[derive(Deserialize, Debug)]
struct NpmPackageInfo {
    #[serde(rename = "dist-tags")]
    dist_tags: DistTags,
}

#[derive(Deserialize, Debug)]
struct DistTags {
    latest: String,
}

const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");
const PACKAGE_NAME: &str = "tella";

pub async fn check_for_updates() {
    match fetch_latest_version().await {
        Ok(latest_version) => {
            if should_update(&latest_version) {
                print_update_notification(&latest_version);
            }
        }
        Err(_) => {
        }
    }
}

async fn fetch_latest_version() -> Result<String, String> {
    let client = reqwest::Client::new();
    let url = format!("https://registry.npmjs.org/{}", PACKAGE_NAME);

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to fetch npm info: {}", e))?;

    let package_info: NpmPackageInfo = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse npm response: {}", e))?;

    Ok(package_info.dist_tags.latest)
}

fn should_update(latest: &str) -> bool {
    let current_parts: Vec<&str> = CURRENT_VERSION.split('.').collect();
    let latest_parts: Vec<&str> = latest.split('.').collect();

    for i in 0..std::cmp::min(current_parts.len(), latest_parts.len()) {
        let current: u32 = current_parts[i].parse().unwrap_or(0);
        let latest_val: u32 = latest_parts[i].parse().unwrap_or(0);

        if latest_val > current {
            return true;
        } else if latest_val < current {
            return false;
        }
    }

    false
}

fn print_update_notification(latest_version: &str) {
    println!();
    println!("{}", "┌─────────────────────────────────────────┐".yellow());
    println!(
        "{}",
        format!("  {} Update available: {} → {} ", "📦".cyan(), CURRENT_VERSION, latest_version.green())
            .yellow()
    );
    println!("{}", "│ Run: tella --upgrade                    │".yellow());
    println!("{}", "└─────────────────────────────────────────┘".yellow());
    println!();
}

pub async fn perform_upgrade() -> Result<(), String> {
    println!("{}", "🔄 Checking for updates...".cyan());
    
    let latest_version = fetch_latest_version().await?;

    if !should_update(&latest_version) {
        println!("{}", "✓ You're already on the latest version!".green());
        return Ok(());
    }

    let install_cmd = "npm add -g tella@latest";

    println!(
        "{}",
        format!(
            "⬆️  Upgrading from {} to {}...",
            CURRENT_VERSION, latest_version
        )
        .cyan()
    );

    #[cfg(target_os = "windows")]
    {
        use std::process::Command;
        Command::new("powershell")
            .args(&["-Command", install_cmd])
            .spawn()
            .map_err(|e| format!("Failed to run upgrade: {}", e))?
            .wait()
            .map_err(|e| format!("Upgrade failed: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        use std::process::Command;
        Command::new("bash")
            .args(&["-c", install_cmd])
            .spawn()
            .map_err(|e| format!("Failed to run upgrade: {}", e))?
            .wait()
            .map_err(|e| format!("Upgrade failed: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        Command::new("bash")
            .args(&["-c", install_cmd])
            .spawn()
            .map_err(|e| format!("Failed to run upgrade: {}", e))?
            .wait()
            .map_err(|e| format!("Upgrade failed: {}", e))?;
    }

    println!("{}", "✓ Upgrade complete!".green());
    Ok(())
}
