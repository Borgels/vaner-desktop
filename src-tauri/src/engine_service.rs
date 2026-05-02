//! systemd-user `vaner-engine.service` install / uninstall flow.
//!
//! Auto-bring-up (the [`bring_up`] module) handles "engine up while the
//! desktop is running". For users who want the engine to come up at
//! login *independently* of the desktop — and survive desktop crashes —
//! a systemd-user unit is the right answer. We intentionally do not
//! ship the unit in the `.deb` postinst because postinst would have to
//! cover every logged-in user / locked sessions / mismatched workspace
//! state. Per-user, opt-in install via Preferences keeps the deb
//! payload simple and gives the user explicit control.
//!
//! Tauri commands:
//!   - [`engine_service_status`] — what the unit looks like right now
//!     (`Missing` / `Disabled` / `Enabled` / `Active`), plus the
//!     linger flag (whether the user-manager survives logout).
//!   - [`engine_service_install`] — write the unit, daemon-reload,
//!     `enable --now`. Refuses if no workspace is set.
//!   - [`engine_service_uninstall`] — `disable --now`, remove the unit
//!     file, daemon-reload. Idempotent (no-op if missing).
//!   - [`engine_service_set_linger`] — toggle `loginctl enable-linger
//!     / disable-linger` for the current user via pkexec (graphical
//!     polkit prompt). Without linger, the user manager exits on
//!     logout and the engine stops with it.

use std::path::{Path, PathBuf};
use std::process::Stdio;

use serde::Serialize;
use tokio::process::Command;

const UNIT_NAME: &str = "vaner-engine.service";

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceState {
    /// systemd is unavailable (no `systemctl`, or `systemctl --user`
    /// returns the "Failed to connect to bus" error common in CI / docker
    /// containers without a user manager). Surfaces a Preferences
    /// hint instead of a broken toggle.
    Unavailable,
    /// Unit file isn't installed. Default state on a fresh launch.
    Missing,
    /// Unit installed but not enabled (won't come up on next login).
    Disabled,
    /// Unit enabled but not currently running.
    Enabled,
    /// Unit enabled and running right now.
    Active,
}

#[derive(Debug, Clone, Serialize)]
pub struct ServiceStatus {
    pub state: ServiceState,
    /// The workspace path baked into the installed unit, if any. Lets
    /// the Preferences pane warn when the user changes their workspace
    /// but the systemd unit still points at the old one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    /// Path to the unit file (whether or not it currently exists).
    pub unit_path: String,
    /// Whether the user manager keeps running after logout. Without
    /// this, an `enabled --now` unit stops as soon as the user logs
    /// out and only restarts when they next log in graphically. The
    /// canonical signal is the existence of
    /// `/var/lib/systemd/linger/<user>`.
    pub linger_enabled: bool,
    /// Human-readable detail for error toasts.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

fn unit_path() -> Option<PathBuf> {
    let base = if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        if !xdg.is_empty() {
            PathBuf::from(xdg)
        } else {
            PathBuf::from(std::env::var("HOME").ok()?).join(".config")
        }
    } else {
        PathBuf::from(std::env::var("HOME").ok()?).join(".config")
    };
    Some(base.join("systemd").join("user").join(UNIT_NAME))
}

async fn systemctl_user(args: &[&str]) -> Result<(bool, String, String), String> {
    let output = Command::new("systemctl")
        .arg("--user")
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("systemctl --user spawn failed: {e}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    Ok((output.status.success(), stdout, stderr))
}

async fn systemctl_available() -> bool {
    if which::which("systemctl").is_err() {
        return false;
    }
    // `systemctl --user --version` is the cheapest reachability probe;
    // it succeeds when there's a user manager bus and fails with
    // "Failed to connect to bus" in containers.
    matches!(
        Command::new("systemctl")
            .args(["--user", "--no-pager", "is-system-running"])
            .output()
            .await,
        Ok(o) if o.status.code().is_some()
    )
}

fn read_workspace_from_unit(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    // `Environment=VANER_PATH=/path/to/repo` is what we write below; if
    // someone hand-edited the unit, fall back to the `--path` arg in
    // ExecStart.
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("Environment=VANER_PATH=") {
            return Some(rest.trim().to_string());
        }
        if line.starts_with("ExecStart=") {
            if let Some(idx) = line.find("--path") {
                let after = &line[idx + "--path".len()..];
                let mut chars = after.chars();
                while let Some(c) = chars.next() {
                    if !c.is_whitespace() {
                        let mut buf = String::from(c);
                        for c2 in chars.by_ref() {
                            if c2.is_whitespace() {
                                break;
                            }
                            buf.push(c2);
                        }
                        return Some(buf);
                    }
                }
            }
        }
    }
    None
}

/// Probe linger via the canonical filesystem signal:
/// `/var/lib/systemd/linger/<user>` is created by `loginctl
/// enable-linger` and removed by `disable-linger`. We check this
/// directly rather than parsing `loginctl show-user` output because
/// the signal exists even on minimal systems where the loginctl
/// binary might be missing or the dbus call would fail in a
/// container.
fn linger_probe() -> bool {
    let Ok(user) = std::env::var("USER") else {
        return false;
    };
    if user.is_empty() {
        return false;
    }
    Path::new("/var/lib/systemd/linger").join(&user).exists()
}

#[tauri::command]
pub async fn engine_service_status() -> Result<ServiceStatus, String> {
    let path = unit_path().ok_or_else(|| "could not resolve $HOME".to_string())?;
    let path_str = path.to_string_lossy().into_owned();
    let linger_enabled = linger_probe();

    if !systemctl_available().await {
        return Ok(ServiceStatus {
            state: ServiceState::Unavailable,
            workspace: None,
            unit_path: path_str,
            linger_enabled,
            detail: Some(
                "systemctl --user is unavailable on this session — the engine will only run while the desktop is open.".to_string(),
            ),
        });
    }

    let workspace = read_workspace_from_unit(&path);
    if !path.exists() {
        return Ok(ServiceStatus {
            state: ServiceState::Missing,
            workspace,
            unit_path: path_str,
            linger_enabled,
            detail: None,
        });
    }

    let (_, is_active, _) = systemctl_user(&["is-active", UNIT_NAME])
        .await
        .unwrap_or((false, "inactive".to_string(), String::new()));
    let (_, is_enabled, _) = systemctl_user(&["is-enabled", UNIT_NAME])
        .await
        .unwrap_or((false, "disabled".to_string(), String::new()));

    let state = match (is_active.as_str(), is_enabled.as_str()) {
        ("active", _) => ServiceState::Active,
        (_, "enabled") => ServiceState::Enabled,
        _ => ServiceState::Disabled,
    };
    Ok(ServiceStatus {
        state,
        workspace,
        unit_path: path_str,
        linger_enabled,
        detail: None,
    })
}

fn render_unit(vaner_bin: &Path, workspace: &str) -> String {
    // KillSignal=SIGINT lets the existing `try: ... except
    // KeyboardInterrupt: down()` block in vaner up handle a graceful
    // shutdown (calls run_down to clean up PID files). systemd's
    // default TERM would skip that handler.
    //
    // Restart=on-failure with a bounded RestartSec catches transient
    // crashes (e.g. ollama not yet ready at boot) without busy-looping
    // when the fault is permanent (bad workspace path → 5s gap is
    // plenty of telemetry headroom).
    format!(
        "[Unit]\n\
Description=Vaner predictive context engine\n\
Documentation=https://vaner.ai/docs\n\
After=network-online.target\n\
\n\
[Service]\n\
Type=simple\n\
Environment=VANER_PATH={workspace}\n\
ExecStart={bin} up --path {workspace}\n\
KillSignal=SIGINT\n\
TimeoutStopSec=15\n\
Restart=on-failure\n\
RestartSec=5\n\
\n\
[Install]\n\
WantedBy=default.target\n",
        bin = vaner_bin.display(),
        workspace = workspace,
    )
}

#[tauri::command]
pub async fn engine_service_install() -> Result<ServiceStatus, String> {
    if !systemctl_available().await {
        return Err(
            "systemctl --user is unavailable on this session; cannot install the engine service.".to_string(),
        );
    }
    let workspace = crate::workspace::resolve()
        .ok_or_else(|| "Pick a workspace before enabling the background engine service.".to_string())?;
    let workspace_str = workspace.to_string_lossy().into_owned();
    let bin = crate::vaner_cli::resolve_vaner_bin()?;

    let path = unit_path().ok_or_else(|| "could not resolve $HOME".to_string())?;
    let dir = path.parent().ok_or_else(|| "unit path has no parent".to_string())?;
    std::fs::create_dir_all(dir).map_err(|e| format!("create unit dir: {e}"))?;

    let body = render_unit(&bin, &workspace_str);
    // Atomic write: tmp + rename, same pattern as workspace::state.json.
    let tmp = path.with_extension("service.tmp");
    std::fs::write(&tmp, body).map_err(|e| format!("write unit: {e}"))?;
    std::fs::rename(&tmp, &path).map_err(|e| format!("rename unit: {e}"))?;

    // daemon-reload before enable so systemd picks up the new file;
    // enable --now starts it immediately. Failures here leave the
    // unit on disk so the user can retry without re-typing anything.
    let (ok_reload, _, err_reload) = systemctl_user(&["daemon-reload"]).await?;
    if !ok_reload {
        return Err(format!("systemctl daemon-reload failed: {err_reload}"));
    }
    let (ok_enable, _, err_enable) = systemctl_user(&["enable", "--now", UNIT_NAME]).await?;
    if !ok_enable {
        return Err(format!("systemctl enable --now failed: {err_enable}"));
    }
    engine_service_status().await
}

#[tauri::command]
pub async fn engine_service_uninstall() -> Result<ServiceStatus, String> {
    let path = unit_path().ok_or_else(|| "could not resolve $HOME".to_string())?;
    if !path.exists() {
        return engine_service_status().await;
    }
    if systemctl_available().await {
        // Best-effort disable; if the unit is somehow already gone
        // from systemd's view we still want to remove the file.
        let _ = systemctl_user(&["disable", "--now", UNIT_NAME]).await;
    }
    if let Err(e) = std::fs::remove_file(&path) {
        return Err(format!("remove unit: {e}"))
            .map_err(|e: String| format!("uninstall: {e}"));
    }
    if systemctl_available().await {
        let _ = systemctl_user(&["daemon-reload"]).await;
    }
    engine_service_status().await
}

/// Toggle `loginctl enable-linger / disable-linger` for the current
/// user. Without linger, the per-user systemd manager exits when the
/// user logs out, taking `vaner-engine.service` with it; the unit
/// only restarts when the user logs back in graphically. With linger
/// enabled, the user manager keeps running across reboots and the
/// engine survives logout. This is the right setting if the user
/// wants Vaner indexing in the background even when they're away.
///
/// `loginctl enable-linger` requires `auth_admin` from polkit by
/// default (action `org.freedesktop.login1.set-self-linger`), so we
/// shell `pkexec` to surface a graphical password prompt via the
/// session's polkit agent. Falls back to a clear error when pkexec
/// isn't installed (some headless distros, containers).
#[tauri::command]
pub async fn engine_service_set_linger(enable: bool) -> Result<ServiceStatus, String> {
    let user = std::env::var("USER").map_err(|_| "USER env var not set".to_string())?;
    if user.is_empty() {
        return Err("USER env var is empty".to_string());
    }

    if which::which("loginctl").is_err() {
        return Err(
            "loginctl is not installed; cannot toggle linger on this system.".to_string(),
        );
    }
    if which::which("pkexec").is_err() {
        return Err(format!(
            "pkexec is required to toggle linger but isn't installed.\n\
             Run this manually instead:\n  sudo loginctl {} {}",
            if enable { "enable-linger" } else { "disable-linger" },
            user
        ));
    }

    let action = if enable { "enable-linger" } else { "disable-linger" };
    let output = Command::new("pkexec")
        .arg("loginctl")
        .arg(action)
        .arg(&user)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("could not spawn pkexec: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        // pkexec exits 126 when the user dismissed the prompt; surface
        // a kinder message than the raw "Authorization failed" so a
        // cancelled click looks like a cancellation, not a bug.
        if output.status.code() == Some(126) {
            return Err("Authorization cancelled.".to_string());
        }
        return Err(if stderr.is_empty() {
            format!(
                "loginctl {} exited with code {}",
                action,
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr
        });
    }

    engine_service_status().await
}
