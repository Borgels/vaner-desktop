//! Thin Ollama HTTP client for the Preferences pane.
//!
//! Talks to the Ollama daemon over loopback (`127.0.0.1:11434`) so
//! the Models card can list installed models, pull new ones (with
//! streaming progress), and remove unwanted ones — without making
//! the user drop into a terminal.
//!
//! Tauri commands:
//!   - [`ollama_list`] — `GET /api/tags`. Returns name + size +
//!     mtime per installed model. Empty list when Ollama is
//!     reachable but has nothing pulled yet; `unavailable=true`
//!     when the daemon isn't reachable at all (different UX —
//!     "install Ollama" vs "pull a model").
//!   - [`ollama_pull(name)`] — streams `POST /api/pull` and emits
//!     `ollama:pull-progress` events while the layers download.
//!     Final event is `ollama:pull-done` with `success` + (on
//!     failure) `detail`. Returns immediately so the UI can show
//!     a sticky progress card without blocking.
//!   - [`ollama_cancel_pull`] — abort the active pull. Idempotent
//!     when nothing is in flight.
//!   - [`ollama_remove(name)`] — `DELETE /api/delete`.
//!
//! The daemon-side `vaner config set backend.model <id>` is the
//! activation surface for "use this model" — not a separate Ollama
//! command, so the existing `set_local_model` Tauri handler covers
//! that flow without any new code here.

use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;

const OLLAMA_BASE: &str = "http://127.0.0.1:11434";

/// Process-wide handle to the in-flight pull task. Wrapped in an
/// Arc<Mutex<>> so `ollama_cancel_pull` can abort whichever task is
/// currently running, regardless of which window initiated it.
static PULL_HANDLE: tokio::sync::OnceCell<Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>>> =
    tokio::sync::OnceCell::const_new();

async fn pull_handle() -> Arc<Mutex<Option<tauri::async_runtime::JoinHandle<()>>>> {
    PULL_HANDLE
        .get_or_init(|| async { Arc::new(Mutex::new(None)) })
        .await
        .clone()
}

#[derive(Debug, Clone, Serialize)]
pub struct InstalledModel {
    pub name: String,
    pub size_bytes: u64,
    /// Human-readable size — "1.9 GiB", "986 MiB", etc. Computed in
    /// Rust so every call site shows the same string.
    pub size_display: String,
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct OllamaListResult {
    pub available: bool,
    pub models: Vec<InstalledModel>,
    /// Human-readable detail for the "unavailable" case: "Ollama is
    /// not running on 127.0.0.1:11434" / "Connection refused" / etc.
    /// Empty when `available=true`.
    pub detail: String,
}

fn human_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB"];
    let mut n = bytes as f64;
    let mut i = 0;
    while n >= 1024.0 && i + 1 < UNITS.len() {
        n /= 1024.0;
        i += 1;
    }
    if i == 0 {
        format!("{n:.0} {}", UNITS[i])
    } else {
        format!("{n:.1} {}", UNITS[i])
    }
}

#[derive(Deserialize)]
struct TagsResponse {
    #[serde(default)]
    models: Vec<TagModel>,
}

#[derive(Deserialize)]
struct TagModel {
    name: String,
    #[serde(default)]
    size: u64,
    modified_at: Option<String>,
}

fn http_client() -> Result<reqwest::Client, String> {
    reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| format!("could not build HTTP client: {e}"))
}

#[tauri::command]
pub async fn ollama_list() -> Result<OllamaListResult, String> {
    let client = http_client()?;
    let url = format!("{OLLAMA_BASE}/api/tags");
    let response = match client.get(&url).send().await {
        Ok(r) => r,
        Err(e) => {
            // Connection refused / DNS / TLS — different reasons,
            // same UX (Ollama unavailable). Surface the kind in
            // detail so the row helps the user decide what to do.
            return Ok(OllamaListResult {
                available: false,
                models: Vec::new(),
                detail: if e.is_connect() {
                    "Ollama is not running on 127.0.0.1:11434.".to_string()
                } else {
                    format!("Could not reach Ollama: {e}")
                },
            });
        }
    };
    if !response.status().is_success() {
        return Ok(OllamaListResult {
            available: false,
            models: Vec::new(),
            detail: format!("Ollama returned HTTP {}.", response.status().as_u16()),
        });
    }
    let parsed: TagsResponse = response
        .json()
        .await
        .map_err(|e| format!("could not parse Ollama tags response: {e}"))?;
    let models = parsed
        .models
        .into_iter()
        .map(|m| InstalledModel {
            size_display: human_bytes(m.size),
            size_bytes: m.size,
            name: m.name,
            modified_at: m.modified_at,
        })
        .collect();
    Ok(OllamaListResult {
        available: true,
        models,
        detail: String::new(),
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct PullProgress {
    /// Model name being pulled. Echoed in every event so a UI
    /// receiving multiple concurrent pulls (we don't allow that
    /// today, but might) can route correctly.
    pub model: String,
    /// Free-form status string from Ollama: "pulling manifest",
    /// "verifying sha256 digest", "downloading layer …", etc.
    pub status: String,
    /// Optional progress fraction `0..=1`. None when the current
    /// step doesn't report total/completed (manifest fetch, etc.).
    pub fraction: Option<f64>,
    pub completed_bytes: Option<u64>,
    pub total_bytes: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PullDone {
    pub model: String,
    pub success: bool,
    /// Empty on success.
    pub detail: String,
}

#[derive(Deserialize)]
struct PullStatusLine {
    status: String,
    #[serde(default)]
    completed: Option<u64>,
    #[serde(default)]
    total: Option<u64>,
}

#[tauri::command]
pub async fn ollama_pull(app: AppHandle, name: String) -> Result<(), String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("model name is required".to_string());
    }
    // Cancel any pull already in flight — Ollama itself doesn't
    // serialise pulls, but the Models card UX assumes one at a
    // time so its progress card stays unambiguous.
    let handle_slot = pull_handle().await;
    {
        let mut guard = handle_slot.lock().await;
        if let Some(prev) = guard.take() {
            prev.abort();
        }
    }

    // Spawn the pull task. Returns immediately; the UI reacts to
    // `ollama:pull-progress` events and the closing `ollama:pull-done`.
    let app_for_task = app.clone();
    let handle_slot_for_task = handle_slot.clone();
    let task_name = trimmed.clone();
    let join = tauri::async_runtime::spawn(async move {
        let result = stream_pull(app_for_task.clone(), &task_name).await;
        let payload = PullDone {
            model: task_name,
            success: result.is_ok(),
            detail: result.err().unwrap_or_default(),
        };
        let _ = app_for_task.emit("ollama:pull-done", &payload);
        let mut guard = handle_slot_for_task.lock().await;
        *guard = None;
    });
    let mut guard = handle_slot.lock().await;
    *guard = Some(join);
    Ok(())
}

async fn stream_pull(app: AppHandle, name: &str) -> Result<(), String> {
    use futures_util::StreamExt as _;
    let client = reqwest::Client::builder()
        // Pulls can run for many minutes; 10-minute idle timeout
        // covers small/medium models on a slow connection. Larger
        // models that exceed this surface as a fail with a clear
        // "connection idle" detail; the user can retry.
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("could not build pull client: {e}"))?;
    let url = format!("{OLLAMA_BASE}/api/pull");
    let body = serde_json::json!({ "name": name, "stream": true });
    let response = client
        .post(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("could not POST /api/pull: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Ollama returned HTTP {} when pulling {name}",
            response.status().as_u16()
        ));
    }

    let mut stream = response.bytes_stream();
    let mut buffer = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("pull stream broke: {e}"))?;
        buffer.extend_from_slice(&chunk);
        // Ollama emits one JSON object per line. Process complete
        // lines; keep the trailing partial line for the next chunk.
        while let Some(idx) = buffer.iter().position(|&b| b == b'\n') {
            let line: Vec<u8> = buffer.drain(..=idx).collect();
            let line_str = String::from_utf8_lossy(&line);
            let trimmed = line_str.trim();
            if trimmed.is_empty() {
                continue;
            }
            let parsed: PullStatusLine = match serde_json::from_str(trimmed) {
                Ok(p) => p,
                Err(_) => continue, // ignore malformed lines
            };
            let fraction = match (parsed.completed, parsed.total) {
                (Some(c), Some(t)) if t > 0 => Some((c as f64 / t as f64).clamp(0.0, 1.0)),
                _ => None,
            };
            let event = PullProgress {
                model: name.to_string(),
                status: parsed.status,
                fraction,
                completed_bytes: parsed.completed,
                total_bytes: parsed.total,
            };
            let _ = app.emit("ollama:pull-progress", &event);
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn ollama_cancel_pull() -> Result<(), String> {
    let handle_slot = pull_handle().await;
    let mut guard = handle_slot.lock().await;
    if let Some(handle) = guard.take() {
        handle.abort();
    }
    Ok(())
}

#[tauri::command]
pub async fn ollama_remove(name: String) -> Result<(), String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("model name is required".to_string());
    }
    let client = http_client()?;
    let url = format!("{OLLAMA_BASE}/api/delete");
    let body = serde_json::json!({ "name": trimmed });
    let response = client
        .delete(&url)
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("could not DELETE /api/delete: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Ollama returned HTTP {} when removing {trimmed}",
            response.status().as_u16()
        ));
    }
    Ok(())
}
