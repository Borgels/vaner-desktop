//! Background SSE subscription. Spawn on app startup; the task lives
//! for the lifetime of the app unless explicitly aborted.
//!
//! Consumes [`vaner_contract::stream_prediction_events`] — which
//! handles backoff, multi-line `data:` frames, and reconnect — and
//! re-emits each snapshot on `predictions:snapshot` via Tauri's event
//! bus. The Svelte `$lib/stores/predictions.ts` store listens.

use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::mpsc;
use vaner_contract::{HttpEngineClient, PredictedPrompt, stream_prediction_events};

pub fn spawn(
    app: AppHandle,
    engine: Arc<HttpEngineClient>,
) -> tauri::async_runtime::JoinHandle<()> {
    // tauri::async_runtime::spawn drives Tauri's managed Tokio runtime;
    // a bare tokio::spawn here panics because the Tauri setup callback
    // is not itself executing inside a Tokio reactor.
    tauri::async_runtime::spawn(async move {
        let (tx, mut rx) = mpsc::channel::<Vec<PredictedPrompt>>(16);
        let _stream_handle = stream_prediction_events(&engine, tx);

        while let Some(snapshot) = rx.recv().await {
            if let Err(e) = app.emit("predictions:snapshot", snapshot) {
                eprintln!("[vaner-desktop] emit predictions:snapshot failed: {e}");
                break;
            }
        }
    })
}
