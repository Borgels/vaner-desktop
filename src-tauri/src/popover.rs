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

pub const WINDOW_LABEL: &str = "main";

/// Show the popover. Window-positioning anchoring (e.g.
/// `tauri-plugin-positioner::Position::TrayCenter`) was tried and
/// removed: the plugin panics with "Tray position not set" when its
/// internal cache is empty, the panic survives `catch_unwind` only on
/// the strict letter — the popover then refuses to surface anyway.
/// Without explicit positioning, the window opens wherever the
/// compositor places it (typically last-known or center-of-screen),
/// which is acceptable for a borderless popover.
pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
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
