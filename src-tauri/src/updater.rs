//! Background updater — checks GitHub Releases via
//! `tauri-plugin-updater`, emits events the Svelte layer can surface
//! as a calm "update available" toast.
//!
//! The plugin handles signature verification against the minisign
//! pubkey baked into `tauri.conf.json`; a tampered `latest.json`
//! fails verification and the check silently returns no update.

use serde::Serialize;
use tauri::{AppHandle, Emitter, Runtime};
use tauri_plugin_updater::UpdaterExt as _;

/// Event payload for `update:available`.
#[derive(Debug, Clone, Serialize)]
pub struct UpdatePayload {
    pub version: String,
    pub current_version: String,
    pub release_notes: Option<String>,
}

/// Kick off a best-effort background update check. Errors are
/// swallowed — the app works fine without the updater, and there's
/// no useful user-facing message for a transient network failure
/// that the user didn't ask about.
pub fn spawn_check<R: Runtime>(app: AppHandle<R>) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = check(app).await {
            // Log at stderr; operators grepping daemon logs will see
            // this, end users won't be bothered.
            eprintln!("[vaner-linux] updater check failed: {e}");
        }
    });
}

async fn check<R: Runtime>(app: AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> {
    let updater = app.updater_builder().build()?;
    let Some(update) = updater.check().await? else {
        return Ok(());
    };

    let payload = UpdatePayload {
        version: update.version.clone(),
        current_version: update.current_version.clone(),
        release_notes: update.body.clone(),
    };

    app.emit("update:available", &payload)?;
    Ok(())
}

/// `#[tauri::command]` — invoked from Svelte when the user clicks
/// "Install update" on the toast. Downloads + installs + emits
/// progress events on `update:progress` for a future UI progress
/// bar; once finished the app is in a restart-required state.
#[tauri::command]
pub async fn install_update<R: Runtime>(app: AppHandle<R>) -> Result<(), String> {
    let updater = app
        .updater_builder()
        .build()
        .map_err(|e| format!("updater init failed: {e}"))?;
    let Some(update) = updater
        .check()
        .await
        .map_err(|e| format!("updater check failed: {e}"))?
    else {
        return Ok(());
    };

    let app_handle = app.clone();
    update
        .download_and_install(
            |chunk, total| {
                let fraction = total
                    .map(|t| (chunk as f64) / (t as f64))
                    .unwrap_or(0.0)
                    .clamp(0.0, 1.0);
                let _ = app_handle.emit("update:progress", fraction);
            },
            || {
                let _ = app_handle.emit("update:ready-to-restart", ());
            },
        )
        .await
        .map_err(|e| format!("download-and-install failed: {e}"))?;
    Ok(())
}
