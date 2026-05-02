//! Compute / engine-config plumbing for the Preferences pane.
//!
//! The cockpit already exposes the live `[compute]` section under
//! `vaner status --json`, and `vaner config set <key> <value>` is the
//! one and only mutation surface — we do not parse / rewrite
//! `.vaner/config.toml` from Rust. Keeping the CLI as the single
//! mutation path means that whatever validation, alias resolution,
//! and default propagation lives in the Python side keeps working;
//! the desktop is just a UX layer.
//!
//! Tauri commands:
//!   - [`compute_config_get`] — read the live `[compute]` block.
//!   - [`compute_config_set`] — `vaner config set` for a single key.
//!   - [`compute_apply_preset`] — three-pack of writes for the
//!     `Light` / `Balanced` / `Performance` presets the macOS app
//!     ships, so a single click sets a coherent default rather than
//!     forcing the user to slide three knobs.

use std::process::Stdio;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComputeConfig {
    /// `auto` / `cpu` / `cuda` / `metal` — the inference target.
    #[serde(default)]
    pub device: String,
    /// 0–1 fraction of total CPU the precompute loop may use.
    #[serde(default)]
    pub cpu_fraction: f64,
    /// 0–1 fraction of total VRAM the precompute loop may use.
    #[serde(default)]
    pub gpu_memory_fraction: f64,
    /// When true, the precompute loop only runs while the host is
    /// idle (per `idle_cpu_threshold` / `idle_gpu_threshold`). When
    /// false, the loop runs continuously up to the cycle ceiling.
    #[serde(default)]
    pub idle_only: bool,
    /// 0–1 cap on host CPU below which the loop counts the host as
    /// "idle". Higher = more aggressive.
    #[serde(default)]
    pub idle_cpu_threshold: f64,
    /// 0–1 cap on host GPU below which the loop counts the GPU as
    /// "idle". Higher = more aggressive.
    #[serde(default)]
    pub idle_gpu_threshold: f64,
    /// Hard ceiling on wall-clock seconds for one precompute cycle.
    /// Anything that runs longer is cancelled and the cycle is
    /// rescheduled.
    #[serde(default)]
    pub max_cycle_seconds: u64,
}

#[tauri::command]
pub async fn compute_config_get() -> Result<ComputeConfig, String> {
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let workspace = crate::workspace::resolve_str();
    let output = Command::new(&bin)
        .arg("status")
        .arg("--json")
        .arg("--path")
        .arg(&workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("could not spawn `vaner status`: {e}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!("vaner status failed: {stderr}"));
    }

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("could not parse status JSON: {e}"))?;
    let compute = parsed
        .get("compute")
        .ok_or_else(|| "vaner status JSON has no `compute` field".to_string())?;
    serde_json::from_value(compute.clone())
        .map_err(|e| format!("could not parse compute block: {e}"))
}

/// Whitelist of keys the desktop is allowed to write through. Prevents
/// accidental writes to keys outside the compute domain (the CLI would
/// happily accept any key, so the gate lives here). Keep in sync with
/// the macOS app's preset writer; new keys land here when a new
/// Preferences row is added.
const ALLOWED_KEYS: &[&str] = &[
    "compute.cpu_fraction",
    "compute.gpu_memory_fraction",
    "compute.idle_only",
    "compute.idle_cpu_threshold",
    "compute.idle_gpu_threshold",
    "compute.max_cycle_seconds",
    "compute.device",
    "backend.name",
    "backend.base_url",
    "backend.model",
    "backend.api_key_env",
];

async fn run_config_set(key: &str, value: &str) -> Result<(), String> {
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let workspace = crate::workspace::resolve_str();
    let output = Command::new(&bin)
        .arg("config")
        .arg("set")
        .arg(key)
        .arg(value)
        .arg("--path")
        .arg(&workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("could not spawn `vaner config set`: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!(
                "vaner config set {key}={value} exited with code {}",
                output.status.code().unwrap_or(-1)
            )
        } else {
            stderr
        });
    }
    Ok(())
}

#[tauri::command]
pub async fn compute_config_set(key: String, value: String) -> Result<ComputeConfig, String> {
    if !ALLOWED_KEYS.contains(&key.as_str()) {
        return Err(format!(
            "{key} is not on the desktop's write-list; \
             use `vaner config set` directly if you mean to set it.",
        ));
    }
    run_config_set(&key, &value).await?;
    compute_config_get().await
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComputePreset {
    /// Quiet: scans take less, idle-gated.
    Light,
    /// Default: roughly the macOS `balanced` preset.
    Balanced,
    /// Loud: continuous, longer cycles.
    Performance,
}

impl ComputePreset {
    /// Per-preset knob values. The presets are framed around the
    /// thing Vaner actually consumes — GPU — rather than CPU.
    /// The model loop runs the LLM (GPU-bound); the Python that
    /// shepherds it is a rounding error on CPU. Pre-fix the presets
    /// gated on `compute.idle_cpu_threshold`, which made "Light
    /// pauses when you're using your machine" mean "Light pauses
    /// when your CPU is busy" — wrong signal. Now the gating knob is
    /// `compute.idle_gpu_threshold`. The CPU fraction is left at a
    /// modest cap purely so a runaway loop can't peg every core in a
    /// pathological worst case; it isn't the constraint that decides
    /// Light vs Balanced vs Performance.
    fn settings(self) -> [(&'static str, &'static str); 6] {
        match self {
            ComputePreset::Light => [
                ("compute.cpu_fraction", "0.15"),
                ("compute.gpu_memory_fraction", "0.25"),
                ("compute.idle_only", "true"),
                ("compute.idle_gpu_threshold", "0.3"),
                ("compute.idle_cpu_threshold", "0.5"),
                ("compute.max_cycle_seconds", "180"),
            ],
            ComputePreset::Balanced => [
                ("compute.cpu_fraction", "0.25"),
                ("compute.gpu_memory_fraction", "0.4"),
                ("compute.idle_only", "true"),
                ("compute.idle_gpu_threshold", "0.5"),
                ("compute.idle_cpu_threshold", "0.7"),
                ("compute.max_cycle_seconds", "300"),
            ],
            ComputePreset::Performance => [
                ("compute.cpu_fraction", "0.5"),
                ("compute.gpu_memory_fraction", "0.6"),
                ("compute.idle_only", "false"),
                ("compute.idle_gpu_threshold", "0.9"),
                ("compute.idle_cpu_threshold", "0.9"),
                ("compute.max_cycle_seconds", "600"),
            ],
        }
    }
}

#[tauri::command]
pub async fn compute_apply_preset(preset: ComputePreset) -> Result<ComputeConfig, String> {
    // Sequential rather than parallel: `vaner config set` rewrites
    // the same config.toml each call, and racing two writes against
    // the file produces undefined merge order. Trade tens-of-ms
    // latency for correctness.
    for (key, value) in preset.settings() {
        run_config_set(key, value).await?;
    }
    compute_config_get().await
}

// ---------------------------------------------------------------------
// Backend (model provider) configuration
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BackendConfig {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub base_url: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub api_key_env: String,
}

#[tauri::command]
pub async fn backend_config_get() -> Result<BackendConfig, String> {
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let workspace = crate::workspace::resolve_str();
    let output = Command::new(&bin)
        .arg("status")
        .arg("--json")
        .arg("--path")
        .arg(&workspace)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .await
        .map_err(|e| format!("could not spawn `vaner status`: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "vaner status failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("could not parse status JSON: {e}"))?;
    let backend = parsed
        .get("backend")
        .ok_or_else(|| "vaner status JSON has no `backend` field".to_string())?;
    serde_json::from_value(backend.clone())
        .map_err(|e| format!("could not parse backend block: {e}"))
}

/// Backend presets surfaced in the Models pane. Each maps to a known
/// `(name, base_url, default_model, api_key_env)` tuple — same set
/// the macOS app ships, so a user with both desktops gets the same
/// behaviour from "Switch to OpenAI" on either side. `Custom` means
/// "leave the existing values; the user is editing them by hand".
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum BackendPreset {
    Ollama,
    Openai,
    Anthropic,
    Custom,
}

impl BackendPreset {
    fn template(self) -> Option<(&'static str, &'static str, &'static str, &'static str)> {
        match self {
            BackendPreset::Ollama => {
                Some(("openai", "http://localhost:11434/v1", "qwen3.5:8b", ""))
            }
            BackendPreset::Openai => Some((
                "openai",
                "https://api.openai.com/v1",
                "gpt-4o-mini",
                "OPENAI_API_KEY",
            )),
            BackendPreset::Anthropic => Some((
                "anthropic",
                "https://api.anthropic.com/v1",
                "claude-sonnet-4-20250514",
                "ANTHROPIC_API_KEY",
            )),
            // `Custom` deliberately writes nothing — the user is
            // editing fields directly via `compute_config_set`.
            BackendPreset::Custom => None,
        }
    }
}

#[tauri::command]
pub async fn backend_apply_preset(preset: BackendPreset) -> Result<BackendConfig, String> {
    if let Some((name, base_url, model, api_key_env)) = preset.template() {
        // Sequential writes — same correctness reasoning as
        // `compute_apply_preset`. Order: name → base_url → model →
        // api_key_env, so a partial failure leaves the remaining
        // fields pointing at the previous backend rather than a
        // half-converted state.
        run_config_set("backend.name", name).await?;
        run_config_set("backend.base_url", base_url).await?;
        run_config_set("backend.model", model).await?;
        run_config_set("backend.api_key_env", api_key_env).await?;
    }
    backend_config_get().await
}

/// Classify a live BackendConfig as one of the named presets — used
/// to highlight the active card in the Models pane.
pub fn classify_backend(b: &BackendConfig) -> BackendPreset {
    if b.base_url.contains("api.openai.com") {
        BackendPreset::Openai
    } else if b.base_url.contains("api.anthropic.com") {
        BackendPreset::Anthropic
    } else if b.base_url.contains("11434") || b.base_url.contains("ollama") {
        BackendPreset::Ollama
    } else {
        BackendPreset::Custom
    }
}

#[tauri::command]
pub fn backend_classify(backend: BackendConfig) -> BackendPreset {
    classify_backend(&backend)
}
