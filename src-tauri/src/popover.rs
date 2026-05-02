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

use std::panic::AssertUnwindSafe;
use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager, PhysicalPosition, Runtime};
use tauri_plugin_positioner::{Position, WindowExt};

pub const WINDOW_LABEL: &str = "main";
static PINNED: AtomicBool = AtomicBool::new(false);

/// Inset from the right edge of the screen for the SNI-fallback
/// anchor. Matches the visual breathing room a stock GNOME panel
/// gives its right-most indicator.
const FALLBACK_INSET_RIGHT_PX: i32 = 12;
/// Inset below the top edge for the SNI-fallback anchor. Picked to
/// clear the GNOME top panel (~30 px) plus a hair so the popover
/// doesn't kiss the panel border.
const FALLBACK_INSET_TOP_PX: i32 = 36;

/// Show the popover, anchored under the tray icon (NSPopover-style).
///
/// Two-stage placement:
///   1. Try `Position::TrayBottomCenter`. On macOS / Windows /
///      X11-with-real-tray-window this hangs the popover directly
///      below the tray icon center, the closest macOS-NSPopover
///      analog. The plugin reads tray bounds from the cache that
///      `tray::install`'s `on_tray_icon_event` keeps fresh.
///   2. If step 1 panics (older plugin versions when the cache is
///      empty) or produces a position near the screen origin
///      (the StatusNotifierItem path GNOME's AppIndicator extension
///      uses doesn't expose tray geometry, so the cache reports
///      `(0, 0)` — TrayBottomCenter then puts the window at
///      `(0 - 210, 0)` which gets clamped to top-left), fall back
///      to a deterministic top-right anchor on the current monitor
///      (right-aligned with a small inset, just below the panel).
///
/// Step 2 is the right answer for ~all Linux desktops we care about:
/// GNOME, KDE, XFCE, Cinnamon — every one of them puts tray icons in
/// the top-right (or bottom-right) panel area, and "top-right of the
/// monitor" is a closer match for "near the tray icon" than
/// "wherever the compositor felt like".
pub fn show<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let window = app
        .get_webview_window(WINDOW_LABEL)
        .ok_or(tauri::Error::WindowNotFound)?;
    window.show()?;

    let tray_anchor_ok = std::panic::catch_unwind(AssertUnwindSafe(|| {
        window.move_window(Position::TrayBottomCenter).is_ok()
    }))
    .unwrap_or(false);

    let bogus_position = window
        .outer_position()
        .map(|p| p.x < 100 && p.y < 100)
        .unwrap_or(true);

    if !tray_anchor_ok || bogus_position {
        // SNI-tray fallback: top-right of the focused monitor with a
        // small inset. `current_monitor` returns the monitor the
        // window currently overlaps, which after `window.show()` is
        // the user's primary monitor — not whichever happens to be
        // index 0.
        if let Ok(Some(monitor)) = window.current_monitor() {
            let m_size = monitor.size();
            let m_pos = monitor.position();
            let win_size = window
                .outer_size()
                .unwrap_or(tauri::PhysicalSize::new(420, 520));
            let x = m_pos.x + m_size.width as i32
                - win_size.width as i32
                - FALLBACK_INSET_RIGHT_PX;
            let y = m_pos.y + FALLBACK_INSET_TOP_PX;
            let _ = window.set_position(PhysicalPosition::new(x, y));
        }
    }

    window.set_always_on_top(PINNED.load(Ordering::Relaxed))?;
    window.set_focus()?;
    Ok(())
}

/// Hide the popover without destroying state. Called from the focus-
/// loss handler and "Pause" flows.
pub fn hide<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if PINNED.load(Ordering::Relaxed) {
        return Ok(());
    }
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

pub fn is_pinned() -> bool {
    PINNED.load(Ordering::Relaxed)
}

pub fn set_pinned<R: Runtime>(app: &AppHandle<R>, pinned: bool) -> tauri::Result<bool> {
    PINNED.store(pinned, Ordering::Relaxed);
    if let Some(window) = app.get_webview_window(WINDOW_LABEL) {
        window.set_always_on_top(pinned)?;
        if pinned {
            window.show()?;
            window.set_focus()?;
        }
    }
    Ok(pinned)
}

pub fn toggle_pinned<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<bool> {
    set_pinned(app, !is_pinned())
}

#[tauri::command]
pub fn popover_toggle_pinned(app: AppHandle) -> Result<bool, String> {
    toggle_pinned(&app).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn popover_is_pinned() -> bool {
    is_pinned()
}
