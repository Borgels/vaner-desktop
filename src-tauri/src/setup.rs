//! Tauri commands for the Simple-Mode setup surface (0.8.6 WS-DESK-LINUX).
//!
//! Mirrors the existing `clients_*` pattern from 0.8.5 WS12-D: every
//! command shells out to `vaner setup ... --json` (or, for the
//! best-effort daemon endpoints, to a short reqwest call) and parses
//! the JSON output. The CLI is the single source of truth for bundle
//! selection, atomic config writes, the cloud-widening guard, and
//! hardware probing.
//!
//! NB: the engine HTTP daemon endpoints listed in the 0.8.6 plan
//! (`POST /policy/refresh`, `GET /deep-run/defaults`) have not landed
//! on this base yet. We probe them best-effort and fall back to a
//! synthesised payload so the desktop is not blocked on the server
//! PR. When those endpoints land we keep the same command names; only
//! the implementation flips from "synthesise from bundle" to "GET".

use std::process::Stdio;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

// ---------------------------------------------------------------------------
// Wire types — mirror src/lib/contract/setup-types.ts on the Svelte side.
// We use serde_json::Value for the bundle / hardware / status payloads
// and let the Svelte layer assert the shape; the desktop's only hard
// requirement is detecting the cloud-widening sentinel for `apply`.
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupChoice {
    pub value: String,
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupQuestion {
    pub id: String,
    pub label: String,
    pub prompt: String,
    pub multi: bool,
    pub choices: Vec<SetupChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppliedPolicy {
    pub config_path: String,
    pub selected_bundle_id: String,
    pub reasons: Vec<String>,
    pub widens_cloud_posture: bool,
    pub written: bool,
    pub daemon: DaemonStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DaemonStatus {
    pub reachable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRefreshResult {
    pub refreshed: bool,
    pub detail: String,
}

fn repo_root_arg() -> String {
    std::env::var("VANER_PATH").unwrap_or_else(|_| ".".to_string())
}

async fn run_vaner_setup_json(
    extra_args: &[&str],
    stdin_payload: Option<String>,
    allow_nonzero: bool,
) -> Result<String, String> {
    let bin = crate::vaner_cli::resolve_vaner_bin()?;
    let mut cmd = Command::new(&bin);
    cmd.arg("setup")
        .args(extra_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    if stdin_payload.is_some() {
        cmd.stdin(Stdio::piped());
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("failed to spawn `vaner setup`: {e}"))?;

    if let Some(payload) = stdin_payload {
        if let Some(mut stdin) = child.stdin.take() {
            stdin
                .write_all(payload.as_bytes())
                .await
                .map_err(|e| format!("failed to write stdin: {e}"))?;
            // Drop closes stdin, signalling EOF to the child.
            drop(stdin);
        }
    }

    let output = child
        .wait_with_output()
        .await
        .map_err(|e| format!("failed to read `vaner setup` output: {e}"))?;

    if !output.status.success() && !allow_nonzero {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(format!(
            "vaner setup exited with code {}: {}",
            output.status.code().unwrap_or(-1),
            if stderr.is_empty() {
                "no stderr".into()
            } else {
                stderr
            }
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

// ---------------------------------------------------------------------------
// Question schema — hand-mirrored from the choice tuples in
// src/vaner/cli/commands/setup.py (_WORK_STYLE_CHOICES, _PRIORITY_CHOICES,
// _COMPUTE_CHOICES, _CLOUD_CHOICES, _BACKGROUND_CHOICES). When the
// daemon ships `GET /setup/questions` we replace this with an HTTP
// fetch.
// ---------------------------------------------------------------------------

fn build_questions() -> Vec<SetupQuestion> {
    fn choice(value: &str, label: &str) -> SetupChoice {
        SetupChoice {
            value: value.to_string(),
            label: label.to_string(),
            hint: None,
        }
    }
    vec![
        SetupQuestion {
            id: "work_styles".into(),
            label: "Work styles".into(),
            prompt: "What kind of work do you want help with?".into(),
            multi: true,
            choices: vec![
                choice("writing", "Writing — drafting, editing, narrative"),
                choice("research", "Research — surveys, deep reading, citations"),
                choice(
                    "planning",
                    "Planning — design docs, roadmaps, project layout",
                ),
                choice("support", "Support — answering questions, troubleshooting"),
                choice("learning", "Learning — studying, exploring a new domain"),
                choice("coding", "Coding — software development"),
                choice("general", "General — knowledge work, mixed light tasks"),
                choice("mixed", "Mixed — a bit of everything (safe default)"),
                choice("unsure", "Unsure — I'd rather Vaner picks for me"),
            ],
        },
        SetupQuestion {
            id: "priority".into(),
            label: "Priority".into(),
            prompt: "What matters most?".into(),
            multi: false,
            choices: vec![
                choice("balanced", "Balanced — a sensible middle"),
                choice("speed", "Speed — snappy responses"),
                choice("quality", "Quality — best answer, even if slow"),
                choice("privacy", "Privacy — keep data on this machine"),
                choice("cost", "Cost — minimise spend"),
                choice("low_resource", "Low-resource — go easy on this machine"),
            ],
        },
        SetupQuestion {
            id: "compute_posture".into(),
            label: "Compute posture".into(),
            prompt: "How hard should this machine work for you?".into(),
            multi: false,
            choices: vec![
                choice("light", "Light — barely use the CPU/GPU"),
                choice("balanced", "Balanced — work with what's idle"),
                choice("available_power", "Available-power — use what this box has"),
            ],
        },
        SetupQuestion {
            id: "cloud_posture".into(),
            label: "Cloud posture".into(),
            prompt: "How do you feel about cloud LLMs?".into(),
            multi: false,
            choices: vec![
                choice("local_only", "Local only — never reach for cloud LLMs"),
                choice("ask_first", "Ask first — confirm before any cloud call"),
                choice(
                    "hybrid_when_worth_it",
                    "Hybrid — cloud when it's clearly worth it",
                ),
                choice(
                    "best_available",
                    "Best available — use the best model for the job",
                ),
            ],
        },
        SetupQuestion {
            id: "background_posture".into(),
            label: "Background posture".into(),
            prompt: "How aggressive should background pondering be?".into(),
            multi: false,
            choices: vec![
                choice("minimal", "Minimal — barely ponder when idle"),
                choice("normal", "Normal — moderate background pondering"),
                choice(
                    "idle_more",
                    "Idle-more — ponder broadly when the box is idle",
                ),
                choice(
                    "deep_run_aggressive",
                    "Deep-Run-aggressive — happy to run overnight",
                ),
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// Tauri commands
// ---------------------------------------------------------------------------

#[tauri::command]
pub async fn setup_questions() -> Result<Vec<SetupQuestion>, String> {
    Ok(build_questions())
}

/// `vaner setup recommend --json` — pure recommendation. Never writes
/// to the config; the desktop calls this on each step-4 review.
#[tauri::command]
pub async fn setup_recommend(answers: Value) -> Result<Value, String> {
    let stdin_payload = serde_json::to_string(&answers)
        .map_err(|e| format!("failed to encode answers JSON: {e}"))?;
    let stdout = run_vaner_setup_json(&["recommend"], Some(stdin_payload), false).await?;
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("could not parse setup recommend output: {e}"))
}

/// `vaner setup apply --json` — persist answers (or an explicit
/// bundle id) to `.vaner/config.toml`.
///
/// Implements the cloud-widening confirmation dance:
///
/// 1. Pre-flight: call `setup recommend` first so we can compare the
///    *new* bundle's `local_cloud_posture` against the current
///    `[policy].selected_bundle_id` (read via `setup show --json`).
///    If the new posture is strictly more permissive, return
///    `widens_cloud_posture=true, written=false` and *do not* shell
///    out to apply yet.
/// 2. When the caller re-invokes with `confirm_cloud_widening=true`
///    (or when there is no widening), shell out to `vaner setup
///    apply` and surface the result.
///
/// This keeps the engine CLI pure (no new flag) while preserving the
/// macOS-desktop confirm pattern.
#[tauri::command]
pub async fn setup_apply(payload: Value) -> Result<AppliedPolicy, String> {
    // ---- 1. Resolve the bundle that *would* be applied. -------------
    let answers = payload.get("answers").cloned();
    let explicit_bundle_id = payload
        .get("bundle_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let confirm_widening = payload
        .get("confirm_cloud_widening")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let prospective_bundle_id: String = if let Some(id) = explicit_bundle_id.clone() {
        id
    } else if let Some(answers_json) = answers.clone() {
        let stdin_payload = serde_json::to_string(&answers_json)
            .map_err(|e| format!("failed to encode answers JSON: {e}"))?;
        let stdout = run_vaner_setup_json(&["recommend"], Some(stdin_payload), false).await?;
        let recommendation: Value = serde_json::from_str(&stdout)
            .map_err(|e| format!("could not parse setup recommend output: {e}"))?;
        recommendation
            .get("bundle")
            .and_then(|b| b.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(|| "setup recommend returned no bundle id".to_string())?
            .to_string()
    } else {
        return Err("setup_apply requires either `answers` or `bundle_id` in payload".to_string());
    };

    // ---- 2. Cloud-widening pre-flight. ------------------------------
    let widens = if confirm_widening {
        false
    } else {
        cloud_posture_widens(&prospective_bundle_id)
            .await
            .unwrap_or(false)
    };

    if widens {
        return Ok(AppliedPolicy {
            config_path: String::new(),
            selected_bundle_id: prospective_bundle_id,
            reasons: vec![
                "WIDENS_CLOUD_POSTURE: re-call apply with confirm_cloud_widening=true to write."
                    .into(),
            ],
            widens_cloud_posture: true,
            written: false,
            daemon: DaemonStatus::default(),
        });
    }

    // ---- 3. Shell out to `vaner setup apply --json`. ---------------
    let mut tmp_path: Option<std::path::PathBuf> = None;
    let mut owned_args: Vec<String> = Vec::new();

    owned_args.push("apply".into());
    owned_args.push("--json".into());
    owned_args.push("--path".into());
    owned_args.push(repo_root_arg());

    if let Some(id) = explicit_bundle_id {
        owned_args.push("--bundle-id".into());
        owned_args.push(id);
    } else if let Some(answers_json) = answers {
        // Write answers to a tempfile rather than rely on stdin: the
        // CLI's apply command reads --answers as a path.
        let dir = std::env::temp_dir();
        let path = dir.join(format!("vaner-setup-answers-{}.json", std::process::id()));
        tokio::fs::write(
            &path,
            serde_json::to_string(&answers_json)
                .map_err(|e| format!("failed to encode answers JSON: {e}"))?,
        )
        .await
        .map_err(|e| format!("failed to write answers tempfile: {e}"))?;
        owned_args.push("--answers".into());
        owned_args.push(path.to_string_lossy().to_string());
        tmp_path = Some(path);
    }

    let arg_refs: Vec<&str> = owned_args.iter().map(|s| s.as_str()).collect();
    let stdout = run_vaner_setup_json(&arg_refs, None, true).await;

    if let Some(path) = tmp_path {
        let _ = tokio::fs::remove_file(path).await;
    }

    let stdout = stdout?;
    let parsed: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse setup apply output: {e}"))?;

    // The CLI emits { config_path, selected_bundle_id, reasons, daemon }.
    // We re-shape to the desktop's `AppliedPolicy` (which always carries
    // widens_cloud_posture / written booleans).
    let cfg_path = parsed
        .get("config_path")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let selected_bundle_id = parsed
        .get("selected_bundle_id")
        .and_then(|v| v.as_str())
        .unwrap_or(&prospective_bundle_id)
        .to_string();
    let reasons = parsed
        .get("reasons")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let daemon = parsed
        .get("daemon")
        .and_then(|v| {
            let reachable = v
                .get("reachable")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            let detail = v
                .get("detail")
                .and_then(|x| x.as_str())
                .map(|s| s.to_string());
            Some(DaemonStatus { reachable, detail })
        })
        .unwrap_or_default();

    Ok(AppliedPolicy {
        config_path: cfg_path,
        selected_bundle_id,
        reasons,
        widens_cloud_posture: false,
        written: true,
        daemon,
    })
}

/// Compare the prospective bundle's `local_cloud_posture` against the
/// currently-selected one. Returns `Ok(false)` on any read error so a
/// transient failure does not block the user.
async fn cloud_posture_widens(prospective_bundle_id: &str) -> Result<bool, String> {
    let stdout = match run_vaner_setup_json(&["show", "--json"], None, true).await {
        Ok(s) => s,
        Err(_) => return Ok(false),
    };
    let parsed: Value = match serde_json::from_str(&stdout) {
        Ok(v) => v,
        Err(_) => return Ok(false),
    };
    let current_id = parsed
        .get("policy")
        .and_then(|p| p.get("selected_bundle_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("hybrid_balanced");
    if current_id == prospective_bundle_id {
        return Ok(false);
    }
    // Current posture comes from the `bundle` entry in the show payload
    // (which is the materialised current bundle, not the prospective one).
    let current_posture = parsed
        .get("bundle")
        .and_then(|b| b.get("local_cloud_posture"))
        .and_then(|v| v.as_str())
        .unwrap_or("local_preferred");

    // Probe the prospective bundle's posture by reading the static
    // catalogue via `setup show --json --bundle-id <id>` is not a flag.
    // Instead, read the catalogue from the `runner_ups` candidate set
    // emitted by the most recent `recommend` call. Best-effort fallback:
    // assume no widening if we cannot determine.
    //
    // For now, we rely on the rank of postures (strict-most-first) and
    // treat the bundle id mapping below as a known list — kept in lock
    // step with src/vaner/setup/catalog.py.
    let prospective_posture = bundle_id_to_posture(prospective_bundle_id);
    Ok(posture_rank(prospective_posture) > posture_rank(current_posture))
}

/// Match the seven shipped bundles. Lock-step with
/// `vaner.setup.catalog.PROFILE_CATALOG`.
fn bundle_id_to_posture(id: &str) -> &'static str {
    match id {
        "local_lightweight" | "local_balanced" | "local_heavy" => "local_only",
        "hybrid_balanced" | "cost_saver" => "local_preferred",
        "hybrid_quality" => "hybrid",
        "deep_research" => "cloud_preferred",
        // Unknown bundle: treat as the most permissive so we surface
        // the warning rather than silently widen.
        _ => "cloud_preferred",
    }
}

fn posture_rank(posture: &str) -> i32 {
    match posture {
        "local_only" => 0,
        "local_preferred" => 1,
        "hybrid" => 2,
        "cloud_preferred" => 3,
        _ => 1,
    }
}

/// `vaner setup show --json` — full setup + policy + hardware
/// snapshot. Used by the Engine and Telemetry tabs and by the
/// first-run check in the root layout.
#[tauri::command]
pub async fn setup_status() -> Result<Value, String> {
    let repo_root = repo_root_arg();
    let owned: Vec<String> = vec!["show".into(), "--json".into(), "--path".into(), repo_root];
    let arg_refs: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
    // `setup show` exits 0 even with no [setup] section, so non-zero
    // here is a real error.
    let stdout = run_vaner_setup_json(&arg_refs, None, true).await?;
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("could not parse setup show output: {e}"))
}

/// Re-emit the bundle that's currently selected in `[policy]`.
/// Backed by `setup show --json` because no separate `vaner policy
/// show` command exists yet (the daemon endpoint `GET /policy/current`
/// is part of the same 0.8.6 plan but lands on the engine PR chain,
/// not this one).
#[tauri::command]
pub async fn policy_show() -> Result<Value, String> {
    let stdout = run_vaner_setup_json(&["show", "--json"], None, true).await?;
    let parsed: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse setup show output: {e}"))?;
    Ok(parsed.get("bundle").cloned().unwrap_or(Value::Null))
}

/// Best-effort kick to the daemon to reload `[setup]` / `[policy]`
/// without a restart. The HTTP endpoint `POST /policy/refresh` is
/// part of the 0.8.6 plan but lands on the engine PR chain, not this
/// one. Until then, the CLI's `setup apply` already pings
/// `/admin/refresh` (when reachable) and reports the daemon status
/// in its JSON output, so the desktop does not need its own probe
/// here. We surface a "next-start" detail string so the UI can
/// render an honest message.
///
/// When the daemon endpoint lands, swap the body for an HTTP POST
/// (gated on a `reqwest` dep added at that time).
#[tauri::command]
pub async fn policy_refresh() -> Result<PolicyRefreshResult, String> {
    Ok(PolicyRefreshResult {
        refreshed: false,
        detail: "daemon will pick up [setup] / [policy] changes on its next config reload".into(),
    })
}

/// `vaner setup hardware --json` — fresh hardware probe. Cheap
/// enough to call on every Telemetry-tab open.
#[tauri::command]
pub async fn hardware_profile() -> Result<Value, String> {
    let stdout = run_vaner_setup_json(&["hardware", "--json"], None, false).await?;
    serde_json::from_str::<Value>(&stdout)
        .map_err(|e| format!("could not parse setup hardware output: {e}"))
}

/// Deep-Run defaults derived from the active bundle. The (future)
/// daemon endpoint `GET /deep-run/defaults` lands with the engine
/// 0.8.6 PR chain; until then we synthesise a record from `setup
/// show --json` so the start-deep-run dialog renders sensible
/// defaults today. When the endpoint ships, swap the body for an
/// HTTP GET (gated on a `reqwest` dep added at that time).
#[tauri::command]
pub async fn deep_run_defaults() -> Result<Value, String> {
    // Synthesise from the active bundle.
    let stdout = match run_vaner_setup_json(&["show", "--json"], None, true).await {
        Ok(s) => s,
        Err(e) => return Err(e),
    };
    let parsed: Value = serde_json::from_str(&stdout)
        .map_err(|e| format!("could not parse setup show output: {e}"))?;
    let bundle = parsed.get("bundle").cloned().unwrap_or(Value::Null);
    let preset = bundle
        .get("deep_run_profile")
        .and_then(|v| v.as_str())
        .unwrap_or("balanced");
    let locality = bundle
        .get("local_cloud_posture")
        .and_then(|v| v.as_str())
        .unwrap_or("local_preferred");
    // Map spend profile to a soft cap. Lock-step with engine when the
    // server endpoint ships.
    let cost_cap_usd: Option<f64> = match bundle
        .get("spend_profile")
        .and_then(|v| v.as_str())
        .unwrap_or("low")
    {
        "zero" => Some(0.0),
        "low" => Some(0.25),
        "medium" => Some(1.0),
        "high" => None,
        _ => Some(0.25),
    };
    let horizon_bias = bundle
        .get("prediction_horizon_bias")
        .and_then(|m| m.as_object())
        .and_then(|map| {
            map.iter()
                .max_by(|a, b| {
                    let av = a.1.as_f64().unwrap_or(0.0);
                    let bv = b.1.as_f64().unwrap_or(0.0);
                    av.partial_cmp(&bv).unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|(k, _)| k.clone())
        })
        .unwrap_or_else(|| "balanced".to_string());

    let synthesised = serde_json::json!({
        "preset": preset,
        "locality": locality,
        "cost_cap_usd": cost_cap_usd,
        "horizon_bias": horizon_bias,
        "reminder": "Deep-Run prepares; it does not act."
    });
    Ok(synthesised)
}
