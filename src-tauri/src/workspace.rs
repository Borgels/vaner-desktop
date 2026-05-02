//! Workspace selection and persistence.
//!
//! The desktop drives the Vaner CLI against a single repo path. Before
//! v0.2.4, that path was resolved per-call as either `$VANER_PATH` or
//! `"."` — which on a `.deb`-installed binary launched from systemd /
//! desktop entry meant `cwd = /`, so the daemon would refuse to start
//! ("non-repo root path") and the popover showed a permanent "Engine
//! unavailable" panel even though everything else was fine.
//!
//! This module persists the user's chosen workspace to
//! `$XDG_CONFIG_HOME/vaner-desktop/state.json` (falling back to
//! `~/.config/vaner-desktop/state.json`) and exposes:
//!
//! - [`resolve`] — single source of truth used by every CLI invocation
//!   shim (`setup.rs`, `diagnostics.rs`, `engine.rs`). Order:
//!   `$VANER_PATH` env override > persisted `state.json` > `None`.
//! - [`workspace_get`] / [`workspace_set`] / [`workspace_pick`] —
//!   Tauri commands the Svelte side calls from the onboarding wizard
//!   and the Preferences pane.
//!
//! When [`resolve`] returns `None` the popover lands on a new state
//! (`needsWorkspace`) that prompts the user to pick a folder rather
//! than firing onboarding or showing engine-error noise.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::oneshot;

const STATE_FILE: &str = "state.json";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesktopState {
    /// Absolute path to the repo the desktop drives the CLI against.
    /// `None` means "user hasn't picked one yet" — the popover surfaces
    /// the picker rather than firing onboarding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workspace: Option<PathBuf>,
}

/// `$XDG_CONFIG_HOME/vaner-desktop` if set, otherwise
/// `~/.config/vaner-desktop`. Created lazily on first write.
fn config_dir() -> Option<PathBuf> {
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            return Some(PathBuf::from(xdg).join("vaner-desktop"));
        }
    }
    let home = std::env::var("HOME").ok()?;
    Some(PathBuf::from(home).join(".config").join("vaner-desktop"))
}

fn state_path() -> Option<PathBuf> {
    config_dir().map(|d| d.join(STATE_FILE))
}

fn read_state() -> DesktopState {
    let Some(path) = state_path() else {
        return DesktopState::default();
    };
    let Ok(text) = std::fs::read_to_string(&path) else {
        return DesktopState::default();
    };
    serde_json::from_str(&text).unwrap_or_default()
}

fn write_state(state: &DesktopState) -> Result<(), String> {
    let Some(dir) = config_dir() else {
        return Err("could not resolve config dir".to_string());
    };
    std::fs::create_dir_all(&dir).map_err(|e| format!("create config dir: {e}"))?;
    let path = dir.join(STATE_FILE);
    let json =
        serde_json::to_string_pretty(state).map_err(|e| format!("serialize state: {e}"))?;
    // Atomic write: tmp + rename. On the off-chance state.json is being
    // read by another instance, the rename guarantees consistency.
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, json).map_err(|e| format!("write state: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename state: {e}"))?;
    Ok(())
}

/// Resolve the active workspace path.
///
/// Order:
/// 1. `$VANER_PATH` (an explicit per-process override; useful for tests
///    and when the desktop is launched from a CLI like `vaner-desktop
///    --workspace /path`, which sets the env var before exec).
/// 2. The persisted `state.json` `workspace` field.
/// 3. `None` — no workspace yet. Caller should surface the picker.
pub fn resolve() -> Option<PathBuf> {
    if let Ok(env_path) = std::env::var("VANER_PATH") {
        if !env_path.is_empty() {
            return Some(PathBuf::from(env_path));
        }
    }
    read_state().workspace
}

/// String-form helper for shelling the CLI. Falls back to `"."` only
/// when nothing is resolvable, matching the old behaviour so existing
/// `--path .` invocations don't crash; callers should prefer
/// [`resolve`] when they want to gate behaviour on "no workspace yet".
pub fn resolve_str() -> String {
    resolve()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|| ".".to_string())
}

fn validate_workspace(path: &Path) -> Result<PathBuf, String> {
    if !path.is_absolute() {
        return Err(format!(
            "workspace path must be absolute (got {})",
            path.display()
        ));
    }
    if !path.exists() {
        return Err(format!("workspace path does not exist: {}", path.display()));
    }
    if !path.is_dir() {
        return Err(format!(
            "workspace path is not a directory: {}",
            path.display()
        ));
    }
    Ok(path.to_path_buf())
}

#[tauri::command]
pub fn workspace_get() -> Option<String> {
    resolve().map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
pub fn workspace_set(path: String) -> Result<String, String> {
    let p = PathBuf::from(&path);
    let validated = validate_workspace(&p)?;
    let mut state = read_state();
    state.workspace = Some(validated.clone());
    write_state(&state)?;
    Ok(validated.to_string_lossy().into_owned())
}

#[tauri::command]
pub async fn workspace_pick(app: AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = oneshot::channel();
    let dialog = app.dialog().clone();
    // pick_folder is async; the closure fires on the dialog thread
    // and the oneshot channel funnels the result back here so we can
    // await it. Sending nothing on cancel means the receiver gets
    // `Err(RecvError)` which we map to Ok(None).
    let mut builder = dialog.file().set_title("Pick a Vaner workspace");
    if let Some(start) = resolve() {
        builder = builder.set_directory(start);
    } else if let Ok(home) = std::env::var("HOME") {
        builder = builder.set_directory(home);
    }
    builder.pick_folder(move |selected| {
        let _ = tx.send(selected);
    });
    let selected = rx.await.map_err(|_| "dialog cancelled".to_string())?;
    let Some(path) = selected else { return Ok(None) };
    let path_buf = path
        .into_path()
        .map_err(|e| format!("could not resolve picked path: {e}"))?;
    let validated = validate_workspace(&path_buf)?;
    let mut state = read_state();
    state.workspace = Some(validated.clone());
    write_state(&state)?;
    Ok(Some(validated.to_string_lossy().into_owned()))
}

/// Apply the persisted workspace to the current process's `VANER_PATH`
/// env var so any helper that hasn't been migrated to [`resolve`] yet
/// still sees a stable path. Called once during app setup.
pub fn export_to_env(_app: &AppHandle) {
    if let Some(p) = read_state().workspace {
        // Don't overwrite an explicit launch-time env var.
        if std::env::var_os("VANER_PATH").is_none() {
            // SAFETY: set_var is unsafe in edition 2024 because it can
            // race with other threads reading the environment. We call
            // this exactly once, very early in setup, before any worker
            // task or CLI shim has read VANER_PATH.
            unsafe {
                std::env::set_var("VANER_PATH", p);
            }
        }
    }
}
