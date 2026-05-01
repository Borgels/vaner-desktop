//! Companion window — second Tauri webview pointing at the SvelteKit
//! `/companion` route. Opened from:
//!   - the tray menu's `Show Companion…` and `Preferences…` items
//!   - the popover footer's `Details` button (via the `open_companion`
//!     Tauri command invoked from JS)
//!
//! Lifecycle: the window is created on demand. If it exists, we just
//! show + focus + emit a navigation event the Svelte side reacts to;
//! creating-then-immediately-closing is wasteful.
//!
//! Two entry points:
//!   - `open_window` — a generic helper callable from any runtime context
//!     (tray.rs is generic over `R: Runtime`, so it can't call a
//!     `#[tauri::command]` directly).
//!   - `open_companion` — the `#[tauri::command]` Svelte invokes, just
//!     forwards to `open_window`.

use tauri::{AppHandle, Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const COMPANION_LABEL: &str = "companion";

/// Open or focus the companion window. `tab` selects the initial pane —
/// one of `prepared`, `sources`, `agents`, `models`, `engine`,
/// `preferences`, `diagnostics`. Defaults to `prepared`.
pub fn open_window<R: Runtime>(app: &AppHandle<R>, tab: Option<String>) -> tauri::Result<()> {
    let tab = tab.unwrap_or_else(|| "prepared".to_string());

    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        // Tell the Svelte side which pane to land on. Listened for in
        // src/routes/companion/+layout.ts.
        let _ = app.emit("companion:navigate", tab);
        return Ok(());
    }

    // SvelteKit's static adapter emits build/companion/index.html when
    // /companion's +layout.ts sets trailingSlash='always'. Tauri loads
    // /companion/ as the directory and the HTML lives at index.html
    // inside; SvelteKit's router sees pathname '/companion/' and
    // resolves it to the /companion route normally.
    let url = format!("companion/?tab={tab}");
    WebviewWindowBuilder::new(app, COMPANION_LABEL, WebviewUrl::App(url.into()))
        .title("Vaner")
        .inner_size(820.0, 560.0)
        .min_inner_size(720.0, 480.0)
        .resizable(true)
        .decorations(true)
        .visible(true)
        .build()?;
    Ok(())
}

pub fn close_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        window.close()?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_companion(app: AppHandle, tab: Option<String>) -> tauri::Result<()> {
    open_window(&app, tab)
}

#[tauri::command]
pub fn close_companion(app: AppHandle) -> tauri::Result<()> {
    close_window(&app)
}
