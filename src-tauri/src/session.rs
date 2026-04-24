//! Session / desktop-environment detection for first-run guidance.
//!
//! Tauri v2's tray icon has a known regression on GNOME/Wayland
//! (upstream issue #14234) — Ubuntu 22.04+ ships the AppIndicator
//! extension by default and mitigates it, but vanilla GNOME does
//! not. This module checks the session type and DE at startup and
//! emits a `setup:appindicator-missing` event so the Svelte UI can
//! show a one-time modal with install guidance.
//!
//! KDE (both X11 and Wayland) works out of the box — no nudge needed.

use tauri::{AppHandle, Emitter};

pub fn first_run_nudge(app: &AppHandle) {
    if is_gnome_wayland_without_appindicator() {
        let _ = app.emit("setup:appindicator-missing", ());
    }
}

fn is_gnome_wayland_without_appindicator() -> bool {
    let session_type = std::env::var("XDG_SESSION_TYPE").unwrap_or_default();
    let current_desktop = std::env::var("XDG_CURRENT_DESKTOP").unwrap_or_default();

    if session_type != "wayland" {
        return false;
    }
    if !current_desktop.contains("GNOME") {
        return false;
    }

    // Ask GNOME Shell via gdbus whether any of the known AppIndicator
    // extensions is enabled. If gdbus isn't available (unexpected on
    // GNOME) or the call fails, assume missing — better to nudge than
    // silently ship without a tray icon.
    match std::process::Command::new("gdbus")
        .args([
            "call",
            "--session",
            "--dest",
            "org.gnome.Shell.Extensions",
            "--object-path",
            "/org/gnome/Shell/Extensions",
            "--method",
            "org.gnome.Shell.Extensions.ListExtensions",
        ])
        .output()
    {
        Ok(out) if out.status.success() => {
            let body = String::from_utf8_lossy(&out.stdout);
            // Known extension IDs (Ubuntu fork + upstream).
            let known_ids = [
                "ubuntu-appindicators@ubuntu.com",
                "appindicatorsupport@rgcjonas.gmail.com",
                "KStatusNotifierItem/AppIndicator",
            ];
            !known_ids.iter().any(|id| body.contains(id))
        }
        _ => true,
    }
}
