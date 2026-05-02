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
//! Geometry persistence: position + size are saved to
//! `~/.config/vaner-desktop/state.json` on resize/move/close, so the
//! window opens where the user left it. First-ever open: centered on
//! the focused monitor (via `Position::Center` from
//! `tauri-plugin-positioner`, which respects multi-monitor setups
//! more reliably than the builder's `.center()` does on Wayland).
//!
//! Two entry points:
//!   - `open_window` — a generic helper callable from any runtime
//!     context (tray.rs is generic over `R: Runtime`, so it can't
//!     call a `#[tauri::command]` directly).
//!   - `open_companion` — the `#[tauri::command]` Svelte invokes,
//!     just forwards to `open_window`.

use tauri::{
    AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, Runtime, WebviewUrl,
    WebviewWindowBuilder, WindowEvent,
};
use tauri_plugin_positioner::{Position, WindowExt};

use crate::workspace::{self, WindowGeometry};

pub const COMPANION_LABEL: &str = "companion";

/// Default size when no geometry is persisted yet. Matches the
/// macOS app's companion baseline; sized for the three-pane shell
/// (sidebar + content + optional timeline column).
const DEFAULT_WIDTH: f64 = 820.0;
const DEFAULT_HEIGHT: f64 = 560.0;

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
    let saved = workspace::companion_geometry();
    let (width, height) = saved
        .map(|g| (g.width as f64, g.height as f64))
        .unwrap_or((DEFAULT_WIDTH, DEFAULT_HEIGHT));

    let window = WebviewWindowBuilder::new(app, COMPANION_LABEL, WebviewUrl::App(url.into()))
        .title("Vaner")
        .inner_size(width, height)
        .min_inner_size(720.0, 480.0)
        .resizable(true)
        .decorations(true)
        // Built invisible so the position can be applied before the
        // user sees the window pop up; we show it explicitly below.
        .visible(false)
        .build()?;

    if let Some(g) = saved {
        // Restore last position; the builder already used the saved
        // size. Validation against current monitor geometry happens
        // in `safe_position` so a window that ended up on a now-
        // disconnected display doesn't open offscreen.
        let pos = safe_position(&window, g);
        let _ = window.set_position(pos);
    } else {
        // First-ever open: centre on the focused monitor. Positioner
        // plugin's Center handles multi-monitor more reliably than
        // builder's `.center()` (which centres on monitor index 0).
        let _ = window.move_window(Position::Center);
    }

    window.show()?;
    window.set_focus()?;

    // Persist geometry whenever the user moves, resizes, or closes
    // the window. `WindowEvent::Moved` and `Resized` fire on the
    // compositor's normal event cadence (one per discrete change,
    // not per-pixel), so saving on every event without throttle is
    // fine — the resulting writes are tiny and infrequent.
    let window_for_handler = window.clone();
    window.on_window_event(move |event| match event {
        WindowEvent::Moved(_) | WindowEvent::Resized(_) | WindowEvent::CloseRequested { .. } => {
            if let Some(g) = current_geometry(&window_for_handler) {
                workspace::save_companion_geometry(g);
            }
        }
        _ => {}
    });

    Ok(())
}

/// Compute the desired position, clamped to the bounds of a currently-
/// connected monitor. Without this, restoring `(x: 9000, y: 200)` from
/// a session where the user had an external display would land the
/// window off-screen on a laptop boot.
fn safe_position<R: Runtime>(
    window: &tauri::WebviewWindow<R>,
    g: WindowGeometry,
) -> PhysicalPosition<i32> {
    let candidate = PhysicalPosition::new(g.x, g.y);
    // If any available monitor contains the candidate point, accept it.
    let monitors = window.available_monitors().unwrap_or_default();
    for m in &monitors {
        let pos = m.position();
        let size = m.size();
        let in_x = candidate.x >= pos.x && candidate.x < pos.x + size.width as i32;
        let in_y = candidate.y >= pos.y && candidate.y < pos.y + size.height as i32;
        if in_x && in_y {
            return candidate;
        }
    }
    // Otherwise fall back to centering on the primary monitor.
    if let Some(primary) = window.primary_monitor().ok().flatten() {
        let pos = primary.position();
        let size = primary.size();
        let win_size = window.outer_size().unwrap_or(PhysicalSize::new(
            DEFAULT_WIDTH as u32,
            DEFAULT_HEIGHT as u32,
        ));
        let x = pos.x + (size.width as i32 - win_size.width as i32) / 2;
        let y = pos.y + (size.height as i32 - win_size.height as i32) / 2;
        return PhysicalPosition::new(x, y);
    }
    candidate
}

fn current_geometry<R: Runtime>(window: &tauri::WebviewWindow<R>) -> Option<WindowGeometry> {
    let pos = window.outer_position().ok()?;
    let size = window.outer_size().ok()?;
    Some(WindowGeometry {
        x: pos.x,
        y: pos.y,
        width: size.width,
        height: size.height,
    })
}

pub fn close_window<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    if let Some(window) = app.get_webview_window(COMPANION_LABEL) {
        if let Some(g) = current_geometry(&window) {
            workspace::save_companion_geometry(g);
        }
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
