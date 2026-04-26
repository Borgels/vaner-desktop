//! System-tray icon + menu. Both left-click and right-click surface
//! the menu — the user's preference on Linux (and KDE users expect
//! it too). macOS's "left-click toggles popover, right-click shows
//! menu" split is not the Linux convention.
//!
//! Menu structure:
//!
//!     ┌──────────────────┐
//!     │  Open Vaner      │  ← popover::show
//!     │  Show Companion… │  ← opens the companion window
//!     ├──────────────────┤
//!     │  Preferences…    │  ← opens companion window on Preferences pane
//!     │  Pause           │  ← disabled until daemon ships POST /engine/pause
//!     ├──────────────────┤
//!     │  Quit            │  ← app.exit(0)
//!     └──────────────────┘
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

use crate::popover;

pub const TRAY_ID: &str = "main";

/// Menu item IDs — stringly-typed per Tauri's API.
const ID_OPEN: &str = "open";
const ID_COMPANION: &str = "companion";
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
            ID_COMPANION => {
                // Routed through the same event the in-popover footer
                // Details button uses, so the Svelte side decides which
                // pane to land on (defaults to Prepared).
                let _ = app.emit("menu:open-companion", Option::<String>::None);
            }
            ID_PREFERENCES => {
                // Open the companion window directly on the Preferences
                // pane. The Svelte side translates the payload into the
                // hash route.
                let _ = app.emit("menu:open-companion", Some("preferences"));
            }
            ID_PAUSE => {
                // Disabled in the menu; this branch should never fire.
                // Left here so the match stays exhaustive over the IDs
                // we own.
                let _ = app.emit("menu:toggle-pause", ());
            }
            ID_QUIT => {
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            // The positioner plugin caches the tray icon's bounds so
            // popover::anchor can land on TrayCenter. Forward every
            // tray event regardless of variant — hover, primary click,
            // secondary click — so the cache is populated by the time
            // a menu item gets clicked.
            tauri_plugin_positioner::on_tray_event(tray.app_handle(), &event);
        })
        .build(app)?;
    Ok(())
}

fn build_menu<R: Runtime>(app: &AppHandle<R>) -> tauri::Result<Menu<R>> {
    let open = MenuItem::with_id(app, ID_OPEN, "Open Vaner", true, None::<&str>)?;
    let companion = MenuItem::with_id(app, ID_COMPANION, "Show Companion…", true, None::<&str>)?;
    let sep1 = PredefinedMenuItem::separator(app)?;
    let prefs = MenuItem::with_id(app, ID_PREFERENCES, "Preferences…", true, None::<&str>)?;
    // Pause is wired in the Svelte UI but the daemon side is not yet
    // implemented (CONTRACT.md `POST /engine/pause` is Tier B). Disable
    // the menu item rather than ship a broken affordance.
    let pause = MenuItem::with_id(app, ID_PAUSE, "Pause (coming soon)", false, None::<&str>)?;
    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItem::with_id(app, ID_QUIT, "Quit", true, None::<&str>)?;

    Menu::with_items(
        app,
        &[&open, &companion, &sep1, &prefs, &pause, &sep2, &quit],
    )
}
