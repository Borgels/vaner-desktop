//! Auto-bring-up of the Vaner engine.
//!
//! Before v0.8.9 the desktop launched, polled `vaner status`, and if
//! the cockpit wasn't reachable showed a permanent "Engine unavailable"
//! panel — leaving the user to click _Restart engine_ manually (which
//! itself was a band-aid, not a fix). The *desktop* is what owns the
//! engine lifecycle now: on launch we probe the cockpit, and if it's
//! down we shell `vaner up --detach --path <workspace>` ourselves and
//! wait until `/healthz` answers.
//!
//! This module exposes:
//!
//! - [`spawn_at_startup`] — fire-and-forget background task the
//!   `tauri::Builder::setup` closure calls. Skips if no workspace has
//!   been picked yet (the popover surfaces the picker).
//! - [`ensure_engine_running`] — async helper the popover's
//!   `Restart engine` flow reuses, so the success path is one
//!   code path and the popover can observe completion via the
//!   returned [`BringUpResult`] instead of guessing.
//! - [`bring_up_engine`] — the matching `#[tauri::command]`.
//!
//! The probe is a 250ms-timeout `GET 127.0.0.1:8473/`. We use the bare
//! root rather than `/healthz` because the cockpit's `GET /` answers
//! 200 on the static index without needing the prediction surface to
//! be ready (`--with-engine` may still be initialising).

use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use tokio::process::Command;

const COCKPIT_HOST: &str = "127.0.0.1";
const COCKPIT_PORT: u16 = 8473;
/// Probe timeout per attempt. Short — the cockpit is loopback so a real
/// answer arrives in single-digit ms; anything longer means it's down.
const PROBE_TIMEOUT: Duration = Duration::from_millis(250);
/// Total budget for `ensure_engine_running` to wait after `vaner up`.
/// 10 seconds covers cold model-runtime warmup (Ollama enumeration,
/// scenario DB open) without leaving the popover hanging forever on a
/// truly broken install.
const STARTUP_BUDGET: Duration = Duration::from_secs(10);
/// Poll interval while we wait for the cockpit to answer post-bringup.
const POLL_INTERVAL: Duration = Duration::from_millis(400);

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BringUpOutcome {
    /// Cockpit was already up — no `vaner up` needed.
    AlreadyRunning,
    /// We shelled `vaner up --detach` and the cockpit answered before
    /// the budget expired.
    Started,
    /// We shelled `vaner up` but the cockpit still wasn't answering
    /// after `STARTUP_BUDGET`. Caller should surface `detail` to the
    /// user (it's already-redacted CLI stderr).
    Failed,
    /// No workspace picked yet — bringup is the user's call once they
    /// finish the picker. Not a failure.
    NoWorkspace,
}

#[derive(Debug, Clone, Serialize)]
pub struct BringUpResult {
    pub outcome: BringUpOutcome,
    /// Resolved workspace path the bringup targeted, if any.
    pub workspace: Option<String>,
    /// Human-readable explanation. Empty for `AlreadyRunning`.
    pub detail: String,
}

/// HTTP probe of the cockpit. Returns true on any 2xx/3xx — the cockpit
/// answers 200 on `/` even before `--with-engine` is fully online, and
/// that's enough for the popover to stop showing the error panel. The
/// daemon-status JSON poll downstream picks up engine readiness on its
/// own cadence.
async fn probe() -> bool {
    let url = format!("http://{COCKPIT_HOST}:{COCKPIT_PORT}/");
    let client = match reqwest::Client::builder().timeout(PROBE_TIMEOUT).build() {
        Ok(c) => c,
        Err(_) => return false,
    };
    matches!(client.get(&url).send().await, Ok(resp) if resp.status().is_success())
}

/// Idempotent. If the cockpit is already up: returns immediately. If
/// no workspace is set: returns `NoWorkspace` (the popover handles it).
/// Otherwise shells `vaner up --detach --path <workspace>` and waits
/// up to `STARTUP_BUDGET` for `/` to answer.
pub async fn ensure_engine_running() -> BringUpResult {
    if probe().await {
        return BringUpResult {
            outcome: BringUpOutcome::AlreadyRunning,
            workspace: crate::workspace::resolve().map(|p| p.to_string_lossy().into_owned()),
            detail: String::new(),
        };
    }

    let Some(workspace) = crate::workspace::resolve() else {
        return BringUpResult {
            outcome: BringUpOutcome::NoWorkspace,
            workspace: None,
            detail: "no workspace selected".to_string(),
        };
    };
    let workspace_str = workspace.to_string_lossy().into_owned();

    let bin = match crate::vaner_cli::resolve_vaner_bin() {
        Ok(p) => p,
        Err(e) => {
            return BringUpResult {
                outcome: BringUpOutcome::Failed,
                workspace: Some(workspace_str),
                detail: e,
            };
        }
    };

    // Fire `vaner up --detach`. The CLI returns once the daemon has
    // forked into the background — we still have to poll the cockpit
    // because daemon process != cockpit listening.
    let output = Command::new(&bin)
        .arg("up")
        .arg("--detach")
        .arg("--path")
        .arg(&workspace_str)
        .output()
        .await;

    let stderr = match output {
        Ok(o) if o.status.success() => String::new(),
        Ok(o) => String::from_utf8_lossy(&o.stderr).trim().to_string(),
        Err(e) => format!("could not spawn `vaner up`: {e}"),
    };

    // Poll until the cockpit answers or the budget runs out. We probe
    // even when `vaner up` reported a non-zero exit, because some
    // failure modes (e.g. cockpit already bound by another instance)
    // still result in a healthy endpoint.
    let deadline = Instant::now() + STARTUP_BUDGET;
    while Instant::now() < deadline {
        if probe().await {
            return BringUpResult {
                outcome: BringUpOutcome::Started,
                workspace: Some(workspace_str),
                detail: String::new(),
            };
        }
        tokio::time::sleep(POLL_INTERVAL).await;
    }

    BringUpResult {
        outcome: BringUpOutcome::Failed,
        workspace: Some(workspace_str),
        detail: if stderr.is_empty() {
            "cockpit did not come up within 10 seconds".to_string()
        } else {
            stderr
        },
    }
}

/// Background task launched from `tauri::Builder::setup`. Runs once
/// at startup. Emits an `engine:bring-up` event with the result so the
/// popover and Diagnostics pane can react. Failures are logged but
/// non-fatal — the popover surfaces the error panel either way.
pub fn spawn_at_startup(app: AppHandle) {
    tauri::async_runtime::spawn(async move {
        let result = ensure_engine_running().await;
        if let BringUpOutcome::Failed = result.outcome {
            eprintln!(
                "[vaner-desktop] engine bring-up failed: {} (workspace={})",
                result.detail,
                result.workspace.as_deref().unwrap_or("<none>")
            );
        }
        let _ = app.emit("engine:bring-up", &result);
    });
}

/// `#[tauri::command]` form of [`ensure_engine_running`]. The popover's
/// `Restart engine` flow calls this instead of `diagnostics_restart_engine`
/// when it wants the observed-success path: receive the structured
/// `BringUpResult`, stop the local "Restarting…" spinner, and let the
/// reducer flip out of `.error` on the next status poll.
#[tauri::command]
pub async fn bring_up_engine() -> Result<BringUpResult, String> {
    Ok(ensure_engine_running().await)
}
