//! Tauri commands for the MCP Clients pane (0.8.5 WS12-D).
//!
//! Unlike the prediction commands in `commands.rs` (which talk HTTP to
//! the local daemon), the `clients_*` commands shell out to the
//! `vaner` CLI's `clients` subcommand introduced in Vaner 0.8.5 WS12-A.
//! That CLI is the single source of truth for per-client config paths,
//! atomic + backup-rotated writes, idempotent merges, and launcher
//! drift detection. All this Rust side does is parse the JSON output.
//!
//! Errors map to short human strings via `human_io_error` /
//! `human_subprocess_error` so the Svelte layer can toast them.

use std::path::Path;
use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

// ---------------------------------------------------------------------------
// Wire types matching `vaner clients --format json`
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedClient {
    pub id: String,
    pub label: String,
    pub kind: String,
    pub status: String, // "configured" / "installed" / "missing"
    pub detected: bool,
    pub configured: bool,
    pub config_path: Option<String>,
    pub detail: String,
}

#[derive(Debug, Deserialize)]
struct DetectResponse {
    clients: Vec<DetectedClient>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WriteResult {
    pub client_id: String,
    pub path: Option<String>,
    pub action: String, // "added" / "updated" / "skipped" / "failed"
    pub backup: Option<String>,
    pub error: Option<String>,
    pub manual_snippet: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WriteResponse {
    results: Vec<WriteResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientDrift {
    pub client_id: String,
    pub label: String,
    pub config_path: Option<String>,
    pub drift: bool,
    pub current_in_config: Option<String>,
    pub expected: String,
    pub detail: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoctorReport {
    pub drift: Vec<ClientDrift>,
    pub drift_count: u32,
    pub fix_command: String,
}

// ---------------------------------------------------------------------------
// Subprocess helpers
// ---------------------------------------------------------------------------

/// Resolve the `vaner` binary, preferring an explicit `VANER_BIN`
/// environment override (useful for the AppImage bundle which can ship
/// the CLI alongside) and falling back to the user PATH.
fn resolve_vaner_bin() -> Result<String, String> {
    if let Ok(explicit) = std::env::var("VANER_BIN") {
        if !explicit.is_empty() {
            return Ok(explicit);
        }
    }
    // Use `which` to resolve from $PATH. This is the same fallback the
    // Vaner Python CLI uses internally via shutil.which.
    let output = std::process::Command::new("which")
        .arg("vaner")
        .output()
        .map_err(|e| format!("could not invoke `which vaner`: {e}"))?;
    if !output.status.success() {
        return Err(
            "Vaner binary not found on PATH. Install Vaner via vaner.ai/install or set $VANER_BIN."
                .into(),
        );
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn run_vaner_clients_json(
    repo_root: &Path,
    extra_args: &[&str],
    allow_nonzero: bool,
) -> Result<String, String> {
    let bin = resolve_vaner_bin()?;
    let mut cmd = Command::new(&bin);
    cmd.arg("clients")
        .args(extra_args)
        .arg("--repo-root")
        .arg(repo_root)
        .arg("--format")
        .arg("json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("failed to run `vaner clients`: {e}"))?;

    if !output.status.success() && !allow_nonzero {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "vaner clients exited with code {}: {}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() {
                "no stderr".into()
            } else {
                stderr
            }
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

/// `vaner clients detect --format json` — list every supported MCP
/// client + its install/configured status on this machine.
#[tauri::command]
pub async fn clients_detect(repo_root: String) -> Result<Vec<DetectedClient>, String> {
    let stdout = run_vaner_clients_json(Path::new(&repo_root), &["detect"], false).await?;
    let resp: DetectResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse clients detect output: {e}"))?;
    Ok(resp.clients)
}

#[tauri::command]
pub async fn clients_install(
    repo_root: String,
    client_id: String,
    force: bool,
) -> Result<Vec<WriteResult>, String> {
    let mut args: Vec<&str> = vec!["install", &client_id];
    if force {
        args.push("--force");
    }
    // Install can exit non-zero on partial failure but still emits a per-
    // client breakdown — pass `allow_nonzero=true` so we surface the rows.
    let stdout = run_vaner_clients_json(Path::new(&repo_root), &args, true).await?;
    let resp: WriteResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse clients install output: {e}"))?;
    Ok(resp.results)
}

#[tauri::command]
pub async fn clients_install_all(
    repo_root: String,
    force: bool,
) -> Result<Vec<WriteResult>, String> {
    let mut args: Vec<&str> = vec!["install", "--all"];
    if force {
        args.push("--force");
    }
    let stdout = run_vaner_clients_json(Path::new(&repo_root), &args, true).await?;
    let resp: WriteResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse clients install --all output: {e}"))?;
    Ok(resp.results)
}

#[tauri::command]
pub async fn clients_uninstall(
    repo_root: String,
    client_id: String,
) -> Result<Vec<WriteResult>, String> {
    let stdout =
        run_vaner_clients_json(Path::new(&repo_root), &["uninstall", &client_id], true).await?;
    let resp: WriteResponse = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse clients uninstall output: {e}"))?;
    Ok(resp.results)
}

/// `vaner clients doctor --format json` — exits non-zero on drift,
/// but the JSON payload is still valid; we tolerate non-zero here.
#[tauri::command]
pub async fn clients_doctor(repo_root: String) -> Result<DoctorReport, String> {
    let stdout = run_vaner_clients_json(Path::new(&repo_root), &["doctor"], true).await?;
    let report: DoctorReport = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse clients doctor output: {e}"))?;
    Ok(report)
}
