//! System-tray icon + menu. Both left-click and right-click surface
//! the menu — the user's preference on Linux (and KDE users expect
//! it too). macOS's "left-click toggles popover, right-click shows
//! menu" split is not the Linux convention.
//!
//! Menu structure:
//!
//! ```text
//! ┌──────────────────┐
//! │  Open Vaner      │  ← popover::show
//! │  Pin window      │  ← keeps the small Vaner window open
//! ├──────────────────┤
//! │  Preferences…    │  ← opens companion window on Preferences pane
//! │  Pause / Resume  │  ← emits menu:toggle-pause; Svelte flips
//! │                  │    the isPaused store + .paused popover
//! ├──────────────────┤
//! │  Quit            │  ← app.exit(0)
//! └──────────────────┘
//! ```
//!
//! `on_tray_icon_event` forwards to `tauri_plugin_positioner::on_tray_event`
//! so the positioner plugin's tray-bounds cache stays populated. Without
//! that, `popover::anchor` would always have to fall through to its
//! TopRight fallback.

use tauri::{
    AppHandle, Emitter, Runtime,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};

use crate::{companion, popover};

pub const TRAY_ID: &str = "main";

/// Menu item IDs — stringly-typed per Tauri's API.
const ID_OPEN: &str = "open";
const ID_PIN: &str = "pin";
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
        .show_menu_on_left_click(true)
        .tooltip("Vaner")
        .on_menu_event(|app, event| match event.id.as_ref() {
            ID_OPEN => {
                let _ = popover::show(app);
            }
            ID_PIN => {
                let _ = popover::toggle_pinned(app);
            }
            ID_PREFERENCES => {
                let _ = companion::open_window(app, Some("preferences".into()));
            }
            ID_PAUSE => {
                // Forward to the Svelte side. The popover's
                // app-state store listens for `menu:toggle-pause`
                // and flips the isPaused flag, which the reducer
                // turns into the .paused popover state.
                let _ = app.emit("menu:toggle-pause", ());
            }
            ID_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // The positioner plugin caches the tray icon's bounds so
            // `Position::TrayCenter` knows where to anchor — without
            // this hook the cache stays empty and any later
            // `move_window(TrayCenter)` panics with "Tray position not
            // set". Call it on every tray event regardless of variant.
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
        })
        .build(app)?;
    Ok(())
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let open = MenuItem::with_id(app, ID_OPEN, "Open Vaner", true, None::<&str>)?;
    let pin = MenuItem::with_id(app, ID_PIN, "Pin / Unpin window", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let prefs = MenuItem::with_id(app, ID_PREFERENCES, "Preferences…", true, None::<&str>)?;
    // UI-level mute toggle. Daemon-side POST /engine/pause is still
    // Tier B; today this just flips an isPaused flag the popover
    // reducer reads to enter the .paused state. Re-wire to the
    // engine endpoint when CONTRACT.md ships it.
    let pause = MenuItem::with_id(app, ID_PAUSE, "Pause / Resume", true, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?;

    Menu::with_items(app, &[&open, &pin, &sep1, &prefs, &pause, &sep2, &quit])
}
