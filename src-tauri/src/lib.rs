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
//! - [`tray`] — system-tray icon + menu.
//! - [`popover`] — show / hide / toggle the borderless popover window.
//! - [`updater`] — background update check via tauri-plugin-updater.
//!
//! The public entry is [`run`], called from `main.rs`.

use std::sync::Arc;
use tauri::{Manager, WindowEvent};
use tokio::sync::Mutex;

use vaner_contract::HttpEngineClient;

pub mod agent_detector;
pub mod clients;
pub mod commands;
pub mod companion;
pub mod diagnostics;
pub mod engine;
pub mod onboarding;
pub mod popover;
pub mod prepared_work_endpoint;
pub mod session;
pub mod setup;
pub mod sse_task;
pub mod tray;
pub mod updater;
pub mod vaner_cli;

/// Process-wide state. A single reqwest-backed HTTP client is shared
/// across every `#[tauri::command]` so connection pooling works.
pub struct AppState {
    pub engine: Arc<HttpEngineClient>,
    /// Handle to the SSE background task; kept so the app can abort
    /// on shutdown / reconnection.
    pub sse_handle: Mutex<Option<tauri::async_runtime::JoinHandle<()>>>,
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
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .manage(state)
        .setup(move |app| {
            // Kick off the SSE snapshot stream; the Svelte store
            // listens on `predictions:snapshot`.
            let handle = sse_task::spawn(app.handle().clone(), engine.clone());
            let app_state = app.state::<AppState>();
            tauri::async_runtime::block_on(async {
                *app_state.sse_handle.lock().await = Some(handle);
            });

            // Install the tray icon + menu ("Open Vaner" /
            // Preferences / Pause / Quit). Menu shows on both left
            // and right click per the documented UX contract.
            tray::install(app.handle())?;

            // First-run guidance: if session+DE can't show tray icons
            // without extra setup, nudge the user now.
            session::first_run_nudge(app.handle());

            // Background update check. Emits `update:available` when
            // a new release is on GitHub and its minisign signature
            // verifies against the pubkey in tauri.conf.json. Failure
            // modes (no network, no update, bad signature) are all
            // silent by design — the user didn't ask.
            updater::spawn_check(app.handle().clone());

            Ok(())
        })
        .on_window_event(|window, event| {
            // Menu-bar behaviour: hide the popover when it loses
            // focus, matching NSPopover semantics. The Svelte layer
            // can still re-show via invoke or tray click.
            if window.label() == popover::WINDOW_LABEL
                && matches!(event, WindowEvent::Focused(false))
                && !popover::is_pinned()
            {
                let _ = popover::hide(window.app_handle());
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::active_predictions,
            commands::prepared_work,
            commands::prepared_work_action,
            commands::adopt_prediction,
            commands::app_quit,
            commands::window_hide,
            updater::install_update,
            popover::popover_toggle_pinned,
            popover::popover_is_pinned,
            diagnostics::diagnostics_status,
            diagnostics::diagnostics_doctor,
            diagnostics::diagnostics_restart_engine,
            diagnostics::diagnostics_upgrade_engine,
            clients::clients_detect,
            clients::clients_install,
            clients::clients_install_all,
            clients::clients_uninstall,
            clients::clients_doctor,
            setup::setup_questions,
            setup::setup_recommend,
            setup::models_recommended,
            setup::setup_apply,
            setup::setup_status,
            setup::policy_show,
            setup::policy_refresh,
            setup::hardware_profile,
            setup::deep_run_defaults,
            companion::open_companion,
            companion::close_companion,
            onboarding::open_onboarding,
            onboarding::close_onboarding,
            engine::engine_status,
            agent_detector::detect_agents,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
