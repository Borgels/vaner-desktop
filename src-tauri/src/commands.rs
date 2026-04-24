//! Tauri commands exposed to the Svelte frontend.
//!
//! Each command is a thin wrapper over a [`vaner_contract::EngineClient`]
//! method. Errors are converted to strings for the `invoke` boundary
//! (Tauri serializes `Err` values as JSON).

use tauri::State;
use tauri_plugin_clipboard_manager::ClipboardExt;
use vaner_contract::{EngineClient, EngineClientError, PredictedPrompt, stash_adopt};

use crate::AppState;

#[tauri::command]
pub async fn active_predictions(
    state: State<'_, AppState>,
) -> Result<Vec<PredictedPrompt>, String> {
    state.engine.active_predictions().await.map_err(human)
}

/// Adopt flow:
///  1. POST `/predictions/{id}/adopt` to the daemon.
///  2. Stash the full Resolution (+ raw bytes for unknown server keys)
///     at `$XDG_STATE_HOME/vaner/pending-adopt.json` via
///     `vaner_contract::stash_adopt`.
///  3. Copy a paste-fallback payload to the clipboard
///     (`predicted_response ?? prepared_briefing ?? intent`).
///  4. Return the short intent string so the frontend can toast it.
#[tauri::command]
pub async fn adopt_prediction(
    state: State<'_, AppState>,
    app: tauri::AppHandle,
    prediction_id: String,
) -> Result<String, String> {
    let (resolution, raw) = state.engine.adopt(&prediction_id).await.map_err(human)?;

    // File-drop on a blocking thread — JSONSerialization + fs::rename
    // shouldn't block the main task.
    let raw_bytes = raw.to_vec();
    let resolution_for_stash = resolution.clone();
    let stash_result =
        tokio::task::spawn_blocking(move || stash_adopt(&resolution_for_stash, &raw_bytes))
            .await
            .map_err(|e| format!("handoff task join error: {e}"))?;
    stash_result.map_err(|e| format!("handoff stash failed: {e}"))?;

    let clipboard_body = resolution
        .predicted_response
        .clone()
        .or_else(|| resolution.prepared_briefing.clone())
        .unwrap_or_else(|| resolution.intent.clone());
    // Writing the clipboard on the main actor is cheap; no detach.
    app.clipboard()
        .write_text(clipboard_body)
        .map_err(|e| format!("clipboard write failed: {e}"))?;

    Ok(resolution.intent)
}

fn human(err: EngineClientError) -> String {
    match err {
        EngineClientError::NotFound => "That prediction is no longer active.".into(),
        EngineClientError::EngineUnavailable => {
            "Vaner can't reach the prediction engine right now.".into()
        }
        EngineClientError::InvalidInput => "Invalid prediction.".into(),
        EngineClientError::Transport(_) => "Vaner is unreachable. Is the daemon running?".into(),
        other => format!("{other}"),
    }
}
