//! Popover window lifecycle — show / hide / toggle, plus
//! focus-loss auto-hide for menu-bar-like behaviour.
//!
//! Anchoring the popover to the tray icon goes through
//! `tauri-plugin-positioner`, which caches tray-icon bounds via a hook
//! we install in [`tray::install`]. When the cache is populated the
//! popover lands directly under the tray icon (X11 only — most
//! Wayland compositors refuse the request and we fall through). When
//! the cache is empty (the user opened the menu before the icon ever
//! reported its bounds, e.g. on a fresh launch) the plugin **panics**
//! rather than returning Err. We catch that panic and fall back to
//! `Position::TopRight`, which is also discoverable, never depends on
//! cached bounds, and is what most users encounter on Wayland anyway.
//!
//! The popover shares the `"main"` webview window declared in
//! `tauri.conf.json` — starting hidden + borderless + skip-taskbar,
//! it only becomes visible after the user clicks the tray icon or
//! the "Open Vaner" menu item.

use std::panic::{AssertUnwindSafe, catch_unwind};
use tauri::{AppHandle, Manager, Runtime, WebviewWindow};
use tauri_plugin_positioner::{Position, WindowExt};

pub const WINDOW_LABEL: &str = "main";

/// Show the popover, anchored to the tray icon when possible.
pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    anchor(&window);
    window.show()?;
    window.set_focus()?;
    Ok(())
}

/// Hide the popover without destroying state. Called from the focus-
/// loss handler and "Pause" flows.
pub fn hide<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.hide()?;
    }
    Ok(())
}

/// Toggle visibility — used by keyboard shortcuts or programmatic
/// triggers. Not currently wired to any UI; left public for L5.1
/// follow-ups.
pub fn toggle<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    if window.is_visible().unwrap_or(false) {
        hide(app)
    } else {
        show(app)
    }
}

/// Try to anchor the window near the tray icon, falling back to
/// top-right when the positioner plugin's tray-bounds cache is empty.
/// All errors and panics are swallowed — anchoring is best-effort.
fn anchor<R: Runtime>(window: &WebviewWindow<R>) {
    // First try TrayCenter. If the plugin panics (cache miss), fall
    // through. AssertUnwindSafe is sound here because the window
    // handle's invariants are not broken by a panic in move_window;
    // the call is a one-shot positioning request that mutates only
    // OS-level window state.
    let tray_attempt = catch_unwind(AssertUnwindSafe(|| {
        let _ = window.move_window(Position::TrayCenter);
    }));
    if tray_attempt.is_ok() {
        return;
    }
    // TopRight does not depend on cached tray bounds and works on
    // every supported compositor.
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let _ = window.move_window(Position::TopRight);
    }));
}
