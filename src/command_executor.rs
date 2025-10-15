use std::process::Command;

#[cfg(target_os = "windows")]
pub async fn execute_command(command: &str) -> Result<String, String> {
    let output = Command::new("powershell")
        .arg("-Command")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        if !stderr.is_empty() {
            return Err(format!("Error: {}", stderr));
        }
    }

    Ok(if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) })
}

#[cfg(target_os = "linux")]
pub async fn execute_command(command: &str) -> Result<String, String> {
    let output = Command::new("bash")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        if !stderr.is_empty() {
            return Err(format!("Error: {}", stderr));
        }
    }

    Ok(if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) })
}

#[cfg(target_os = "macos")]
pub async fn execute_command(command: &str) -> Result<String, String> {
    let output = Command::new("sh")
        .arg("-c")
        .arg(command)
        .output()
        .map_err(|e| format!("Failed to execute command: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if !output.status.success() {
        if !stderr.is_empty() {
            return Err(format!("Error: {}", stderr));
        }
    }

    Ok(if stderr.is_empty() { stdout } else { format!("{}\n{}", stdout, stderr) })
}
