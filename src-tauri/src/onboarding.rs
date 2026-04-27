//! Onboarding window — third Tauri webview pointing at the SvelteKit
//! `/onboarding` route. Opened on first launch when the setup status's
//! `completed_at` is null, or via `--onboarding` arg flag for testing.
//!
//! Lifecycle: created lazily, closed on completion (the `onboarding:done`
//! event from the Svelte side calls `close_onboarding`).

use tauri::{AppHandle, Manager, Runtime, WebviewUrl, WebviewWindowBuilder};

pub const ONBOARDING_LABEL: &str = "onboarding";

pub fn open_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(ONBOARDING_LABEL) {
        window.show()?;
        window.unminimize()?;
        window.set_focus()?;
        return Ok(());
    }

    // /onboarding's +layout.ts sets trailingSlash='always', so the
    // static adapter emits build/onboarding/index.html. Loading
    // "onboarding/" (with trailing slash) lets SvelteKit's router
    // see pathname '/onboarding' and resolve it to the right route.
    WebviewWindowBuilder::new(app, ONBOARDING_LABEL, WebviewUrl::App("onboarding/".into()))
        .title("Welcome to Vaner")
        .inner_size(720.0, 540.0)
        .min_inner_size(640.0, 480.0)
        .resizable(false)
        .decorations(true)
        .visible(true)
        .build()?;
    Ok(())
}

pub fn close_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(ONBOARDING_LABEL) {
        window.close()?;
    }
    Ok(())
}

#[tauri::command]
pub fn open_onboarding(app: AppHandle) -> tauri::Result<()> {
    open_window(&app)
}

#[tauri::command]
pub fn close_onboarding(app: AppHandle) -> tauri::Result<()> {
    close_window(&app)
}
