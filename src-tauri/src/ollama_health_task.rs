//! Single-source poller for Ollama presence + reachability.
//!
//! Vaner's local-first default backend is Ollama on
//! `127.0.0.1:11434`. Without it, the model loop 502s on every MCP
//! request and the user has no idea why. The popover therefore needs
//! a dedicated `.ollamaMissing` state — separate from the engine
//! state, since "engine running but Ollama down" is a real failure
//! mode that "engine unreachable" doesn't capture.
//!
//! Same architecture as [`crate::engine_status_task`]: one tokio
//! task probes the world, caches the latest snapshot, and emits an
//! `ollama:health` event when anything changes. Every webview reads
//! through the cache via the [`ollama_health`] Tauri command and
//! listens to the event for live updates — no per-window probe.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;

use crate::ollama::ollama_list;

const POLL_INTERVAL: Duration = Duration::from_secs(15);

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct OllamaHealth {
    /// True when an `ollama` binary is on PATH (or one of the
    /// canonical install dirs probed by [`probe_installed`]). On
    /// Linux this is the canonical signal that Vaner can ask the
    /// daemon to start a model — and conversely, the canonical
    /// signal we should route the popover to `.ollamaMissing`.
    pub installed: bool,
    /// True when an HTTP probe of `127.0.0.1:11434/api/tags` succeeded.
    /// Implies `installed=true` (the daemon can't run if the binary
    /// isn't there) but the converse isn't true — the daemon may be
    /// installed but stopped, e.g. on a fresh install before
    /// `systemctl --user start ollama`.
    pub running: bool,
    /// Optional human-readable hint for the popover. Empty when
    /// healthy. Surfaces as a sub-line in `OllamaMissing.svelte`.
    pub detail: String,
}

#[derive(Debug)]
pub struct OllamaHealthCache {
    snapshot: RwLock<OllamaHealth>,
    notify: tokio::sync::Notify,
}

impl Default for OllamaHealthCache {
    fn default() -> Self {
        Self::new()
    }
}

impl OllamaHealthCache {
    pub fn new() -> Self {
        Self {
            snapshot: RwLock::new(OllamaHealth::default()),
            notify: tokio::sync::Notify::new(),
        }
    }

    pub async fn snapshot(&self) -> OllamaHealth {
        self.snapshot.read().await.clone()
    }

    /// Wake the poll loop immediately. Used after an install to
    /// flip the popover out of `.ollamaMissing` without waiting up
    /// to a full interval for the next tick.
    pub fn refresh(&self) {
        self.notify.notify_one();
    }
}

fn probe_installed() -> bool {
    if which::which("ollama").is_ok() {
        return true;
    }
    // GUI processes (AppImage, .desktop autostart) inherit a
    // sanitised PATH that often excludes `~/.local/bin` and
    // `/usr/local/bin` even when the user installed Ollama there.
    // Probe the canonical install dirs so we don't surface a
    // bogus "Ollama missing" banner.
    let mut candidates: Vec<std::path::PathBuf> = Vec::new();
    if let Some(home) = std::env::var_os("HOME") {
        candidates.push(std::path::PathBuf::from(&home).join(".local/bin/ollama"));
    }
    candidates.extend([
        std::path::PathBuf::from("/usr/local/bin/ollama"),
        std::path::PathBuf::from("/usr/bin/ollama"),
        std::path::PathBuf::from("/opt/homebrew/bin/ollama"),
    ]);
    candidates.iter().any(|p| p.is_file())
}

async fn probe_health() -> OllamaHealth {
    let installed = probe_installed();
    if !installed {
        return OllamaHealth {
            installed: false,
            running: false,
            detail: "Ollama isn't installed.".into(),
        };
    }
    match ollama_list().await {
        Ok(list) if list.available => OllamaHealth {
            installed: true,
            running: true,
            detail: String::new(),
        },
        Ok(list) => OllamaHealth {
            installed: true,
            running: false,
            detail: if list.detail.is_empty() {
                "Ollama isn't responding.".into()
            } else {
                list.detail
            },
        },
        Err(_) => OllamaHealth {
            installed: true,
            running: false,
            detail: "Ollama isn't responding.".into(),
        },
    }
}

pub fn spawn(app: AppHandle, cache: Arc<OllamaHealthCache>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let next = probe_health().await;
            let changed = {
                let mut guard = cache.snapshot.write().await;
                let differs = *guard != next;
                *guard = next.clone();
                differs
            };
            if changed {
                let _ = app.emit("ollama:health", &next);
            }
            tokio::select! {
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
                _ = cache.notify.notified() => {}
            }
        }
    });
}

#[tauri::command]
pub async fn ollama_health(
    cache: tauri::State<'_, Arc<OllamaHealthCache>>,
) -> Result<OllamaHealth, String> {
    Ok(cache.snapshot().await)
}

/// Open a terminal and run `curl -fsSL https://ollama.com/install.sh
/// | sh`. We can't pipe-curl into bash silently from Tauri because the
/// official installer prompts for sudo (it installs a systemd unit);
/// the user needs a real terminal for the prompt. The first terminal
/// emulator that matches the user's machine is launched. After the
/// command exits, the cache is poked so the popover flips immediately.
#[tauri::command]
pub async fn install_ollama(cache: tauri::State<'_, Arc<OllamaHealthCache>>) -> Result<(), String> {
    let inner = "curl -fsSL https://ollama.com/install.sh | sh; \
                 echo; echo 'Ollama installer finished. Press Enter to close.'; \
                 read -r _";
    let candidates: &[(&str, &[&str])] = &[
        ("gnome-terminal", &["--", "bash", "-c"]),
        ("konsole", &["-e", "bash", "-c"]),
        ("xfce4-terminal", &["-e"]),
        ("kitty", &["bash", "-c"]),
        ("alacritty", &["-e", "bash", "-c"]),
        ("foot", &["bash", "-c"]),
        ("xterm", &["-e", "bash", "-c"]),
    ];
    let mut last_err: Option<String> = None;
    for (term, prefix) in candidates {
        if which::which(term).is_err() {
            continue;
        }
        let mut argv: Vec<String> = prefix.iter().map(|s| s.to_string()).collect();
        argv.push(inner.to_string());
        match tokio::process::Command::new(term).args(&argv).spawn() {
            Ok(_) => {
                cache.refresh();
                return Ok(());
            }
            Err(e) => last_err = Some(format!("{term}: {e}")),
        }
    }
    Err(format!(
        "could not launch a terminal for the Ollama installer ({}). \
         Open https://ollama.com/download manually.",
        last_err.unwrap_or_else(|| "no supported terminal found".into())
    ))
}
