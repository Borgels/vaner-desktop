use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

async fn run_vaner(args: &[&str], allow_nonzero: bool) -> Result<String, String> {
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let output = Command::new(&bin)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("failed to run vaner: {e}"))?;
    if !output.status.success() && !allow_nonzero {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "vaner exited with code {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr
        });
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

#[tauri::command]
pub async fn diagnostics_status() -> Result<Value, String> {
    let stdout = run_vaner(&["status", "--json"], true).await?;
    serde_json::from_str::<Value>(&stdout).map_err(|e| format!("could not parse status JSON: {e}"))
}

#[tauri::command]
pub async fn diagnostics_doctor() -> Result<Value, String> {
    let stdout = run_vaner(&["doctor", "--json"], true).await?;
    serde_json::from_str::<Value>(&stdout).map_err(|e| format!("could not parse doctor JSON: {e}"))
}

#[tauri::command]
pub async fn diagnostics_restart_engine() -> Result<String, String> {
    let _ = run_vaner(&["down"], true).await;
    run_vaner(&["up", "--detach"], true)
        .await
        .map(|_| "Vaner restart requested.".to_string())
}

#[tauri::command]
pub async fn diagnostics_upgrade_engine() -> Result<String, String> {
    run_vaner(&["upgrade"], true)
        .await
        .map(|_| "Vaner engine update finished.".to_string())
}
