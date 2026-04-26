//! Popover window lifecycle — show / hide / toggle, plus
//! focus-loss auto-hide for menu-bar-like behaviour.
//!
//! On X11 the positioner plugin places the window right below the
//! tray icon center (mirrors macOS `NSPopover` attachment). On
//! Wayland the compositor can refuse fine-grained window-positioning
//! requests; the fallback is a top-right placement, still discoverable
//! and still borderless.
//!
//! The popover shares the `"main"` webview window declared in
//! `tauri.conf.json` — starting hidden + borderless + skip-taskbar,
//! it only becomes visible after the user clicks the tray icon or
//! the "Open Vaner" menu item.

use tauri::{AppHandle, Manager, Runtime};
use tauri_plugin_positioner::{Position, WindowExt};

pub const WINDOW_LABEL: &str = "main";

/// Show the popover, anchored to the tray icon when possible.
pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    // Anchor to tray. `move_window(TrayCenter)` panics — does not
    // return Err — when the positioner plugin's tray-bounds cache is
    // empty (e.g. on launch via the menu before any tray event has
    // fired). Catch the panic and fall back to TopRight, which is
    // also discoverable and never depends on cached bounds.
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _ = window.move_window(Position::TrayCenter);
    }))
    .or_else(|_| {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = window.move_window(Position::TopRight);
        }))
    });
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
