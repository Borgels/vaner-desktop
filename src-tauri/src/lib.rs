//! Tauri v2 app entry point.
//!
//! Wires the shared [`vaner_contract`] HTTP/SSE client into a
//! Tauri-native runtime. Responsibilities are split across modules:
//!
//! - [`commands`] — `#[tauri::command]` handlers exposed to Svelte via
//!   `invoke()` (active predictions, adopt flow).
//! - [`sse_task`] — background tokio task that subscribes to the
//!   daemon's SSE stream and emits `predictions:snapshot` events to
//!   the WebView.
//! - [`session`] — XDG session / DE detection for first-run guidance
//!   on GNOME/Wayland without the AppIndicator extension.
//! - [`tray`] — system-tray setup (L5 — TODO).
//!
//! The public entry is [`run`], called from `main.rs`.

use std::sync::Arc;
use tokio::sync::Mutex;

use vaner_contract::HttpEngineClient;

pub mod commands;
pub mod session;
pub mod sse_task;

/// Process-wide state. A single reqwest-backed HTTP client is shared
/// across every `#[tauri::command]` so connection pooling works.
pub struct AppState {
    pub engine: Arc<HttpEngineClient>,
    /// Handle to the SSE background task; kept so the app can abort
    /// on shutdown / reconnection.
    pub sse_handle: Mutex<Option<tokio::task::JoinHandle<()>>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            engine: Arc::new(HttpEngineClient::localhost()),
            sse_handle: Mutex::new(None),
        }
    }
}

/// App entry. Called from both `main.rs` and mobile wrappers.
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let state = AppState::default();
    let engine = state.engine.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(state)
        .setup(move |app| {
            // Kick off the SSE snapshot stream; the Svelte store
            // listens on `predictions:snapshot`.
            let handle = sse_task::spawn(app.handle().clone(), engine.clone());
            let app_state = app.state::<AppState>();
            tauri::async_runtime::block_on(async {
                *app_state.sse_handle.lock().await = Some(handle);
            });

            // First-run guidance: if session+DE can't show tray icons
            // without extra setup, nudge the user now.
            session::first_run_nudge(app.handle());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::active_predictions,
            commands::adopt_prediction,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
