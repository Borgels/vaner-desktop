//! System-tray icon + menu. Both left-click and right-click surface
//! the menu — the user's preference on Linux (and KDE users expect
//! it too). macOS's "left-click toggles popover, right-click shows
//! menu" split is not the Linux convention.
//!
//! Menu structure:
//!
//!     ┌──────────────────┐
//!     │  Open Vaner      │  ← toggles the popover
//!     ├──────────────────┤
//!     │  Preferences…    │  ← emits menu:open-preferences event
//!     │  Pause           │  ← emits menu:toggle-pause event
//!     ├──────────────────┤
//!     │  Quit            │  ← app.exit(0)
//!     └──────────────────┘

use tauri::{
    AppHandle, Emitter, Manager, Runtime,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{TrayIconBuilder, TrayIconEvent},
};

use crate::popover;

pub const TRAY_ID: &str = "main";

/// Menu item IDs — stringly-typed per Tauri's API.
const ID_OPEN: &str = "open";
const ID_PREFERENCES: &str = "preferences";
const ID_PAUSE: &str = "pause";
const ID_QUIT: &str = "quit";

/// Install the tray icon. Call from the Tauri `setup` closure.
pub fn install<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<()> {
    let menu = build_menu(app)?;

    let _tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(app.default_window_icon().cloned().ok_or_else(|| {
            tauri::Error::AssetNotFound(
                "default window icon must be present to build the tray icon".into(),
            )
        })?)
        .menu(&menu)
        .menu_on_left_click(true)
        .tooltip("Vaner")
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_OPEN => {
                let _ = popover::show(app);
            }
            ID_PREFERENCES => {
                // Preferences window TBD in a follow-up; for now the
                // Svelte side shows a "coming soon" toast when it
                // hears this.
                let _ = app.emit("menu:open-preferences", ());
            }
            ID_PAUSE => {
                // Pause state lives in the Svelte store for now; the
                // Rust side will forward to the daemon when pause is
                // wired (CONTRACT.md POST /engine/pause is Tier B).
                let _ = app.emit("menu:toggle-pause", ());
            }
            ID_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { .. } = event {
                // `menu_on_left_click(true)` means the menu pops on
                // primary-button clicks for free. We also nudge the
                // popover toward the tray icon so that when the user
                // picks "Open Vaner" it appears anchored.
                use tauri_plugin_positioner::{Position, WindowExt};
                if let Some(window) = tray.app_handle().get_webview_window(popover::WINDOW_LABEL) {
                    let _ = window.move_window(Position::TrayCenter);
                }
            }
        })
        .build(app)?;
    Ok(())
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let open = MenuItem::with_id(app, ID_OPEN, "Open Vaner", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let prefs = MenuItem::with_id(app, ID_PREFERENCES, "Preferences…", true, None::<&str>)?;
    let pause = MenuItem::with_id(app, ID_PAUSE, "Pause", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&open, &sep1, &prefs, &pause, &sep2, &quit])
}
