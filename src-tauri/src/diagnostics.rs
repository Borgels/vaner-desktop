use serde_json::Value;
use std::process::Stdio;
use tokio::process::Command;

/// Repository root the desktop should target when shelling the CLI. The
/// desktop process inherits its cwd from however it was launched (often
/// `/` for `.deb` installs), so commands like `vaner up` would otherwise
/// start a daemon for the wrong workspace and the popover stays in the
/// error state even after a successful "Restart engine" click. Matches
/// setup.rs's `repo_root_arg()` convention so all CLI invocations point
/// at the same workspace; a proper workspace picker is the better fix.
fn repo_root_arg() -> String {
    std::env::var("VANER_PATH").unwrap_or_else(|_| ".".to_string())
}

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
    let path = repo_root_arg();
    let _ = run_vaner(&["down", "--path", &path], true).await;
    run_vaner(&["up", "--detach", "--path", &path], true)
        .await
        .map(|_| "Vaner restart requested.".to_string())
}

#[tauri::command]
pub async fn diagnostics_upgrade_engine() -> Result<String, String> {
    run_vaner(&["upgrade"], true)
        .await
        .map(|_| "Vaner engine update finished.".to_string())
}

/// Persist a local-model override via `vaner config set backend.model <id>`.
/// Called by the Light/Medium/Heavy switcher in the recommended-setup card
/// after `setup_apply` has written the policy bundle. The CLI handles
/// loading/persisting `.vaner/config.toml` so we don't touch it from Rust.
#[tauri::command]
pub async fn set_local_model(model_id: String) -> Result<String, String> {
    if model_id.trim().is_empty() {
        return Err("model_id is required".to_string());
    }
    run_vaner(&["config", "set", "backend.model", &model_id], true)
        .await
        .map(|_| format!("backend.model set to {model_id}"))
}
