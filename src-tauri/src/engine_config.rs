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
const ALLOWED_COMPUTE_KEYS: &[&str] = &[
    "compute.cpu_fraction",
    "compute.gpu_memory_fraction",
    "compute.idle_only",
    "compute.idle_cpu_threshold",
    "compute.idle_gpu_threshold",
    "compute.max_cycle_seconds",
    "compute.device",
];

#[tauri::command]
pub async fn compute_config_set(key: String, value: String) -> Result<ComputeConfig, String> {
    if !ALLOWED_COMPUTE_KEYS.contains(&key.as_str()) {
        return Err(format!(
            "{key} is not on the desktop's compute write-list; \
             use `vaner config set` directly if you mean to set it.",
        ));
    }
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let workspace = crate::workspace::resolve_str();
    let output = Command::new(&bin)
        .arg("config")
        .arg("set")
        .arg(&key)
        .arg(&value)
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
    fn settings(self) -> [(&'static str, &'static str); 5] {
        match self {
            ComputePreset::Light => [
                ("compute.cpu_fraction", "0.15"),
                ("compute.gpu_memory_fraction", "0.25"),
                ("compute.idle_only", "true"),
                ("compute.idle_cpu_threshold", "0.5"),
                ("compute.max_cycle_seconds", "180"),
            ],
            ComputePreset::Balanced => [
                ("compute.cpu_fraction", "0.25"),
                ("compute.gpu_memory_fraction", "0.4"),
                ("compute.idle_only", "true"),
                ("compute.idle_cpu_threshold", "0.6"),
                ("compute.max_cycle_seconds", "300"),
            ],
            ComputePreset::Performance => [
                ("compute.cpu_fraction", "0.5"),
                ("compute.gpu_memory_fraction", "0.6"),
                ("compute.idle_only", "false"),
                ("compute.idle_cpu_threshold", "0.6"),
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
    }
    compute_config_get().await
}
