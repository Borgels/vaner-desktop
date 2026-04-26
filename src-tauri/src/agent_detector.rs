//! Linux equivalent of the macOS AgentDetector. Scans `/proc/*/comm`
//! for known AI-agent process names (Cursor, Claude Desktop, VS Code
//! with Continue, Zed, etc.) and returns a snapshot the reducer
//! consumes via the `anyAgentRunning` flag.
//!
//! `/proc` is the canonical Linux way to enumerate processes without
//! shelling out; it's cheap (one readdir + one read per pid) and
//! requires no special privileges.

use serde::Serialize;
use std::fs;

#[derive(Debug, Clone, Serialize)]
pub struct AgentSuggestionOut {
    pub id: String,
    pub display_name: String,
    pub bundle_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AgentDetectorOut {
    pub running_count: u32,
    pub suggestions: Vec<AgentSuggestionOut>,
}

/// Map a `/proc/*/comm` value (the binary's basename, truncated to 15
/// chars by the kernel) to a friendly agent label. Returns Some(id)
/// when we recognise it, None otherwise.
fn classify(comm: &str) -> Option<&'static str> {
    let lower = comm.to_lowercase();
    // Exact-or-prefix matches; /proc/*/comm is truncated to 15 chars
    // so we can't rely on full process names like "Claude Desktop".
    match lower.as_str() {
        "cursor" => Some("cursor"),
        "claude" => Some("claude-desktop"),
        "code" => Some("vscode"),
        "code-insiders" => Some("vscode-insiders"),
        "zed" => Some("zed"),
        "zeditor" => Some("zed"),
        "continue" => Some("continue"),
        _ => None,
    }
}

#[tauri::command]
pub fn detect_agents() -> Result<AgentDetectorOut, String> {
    #[cfg(not(target_os = "linux"))]
    {
        return Ok(AgentDetectorOut { running_count: 0, suggestions: known_suggestions() });
    }

    #[cfg(target_os = "linux")]
    {
        let mut hits: std::collections::BTreeSet<&'static str> = Default::default();
        let entries = fs::read_dir("/proc").map_err(|e| format!("read /proc: {e}"))?;
        for entry in entries.flatten() {
            let path = entry.path();
            // Only numeric pid dirs.
            let pid_ok = path
                .file_name()
                .and_then(|n| n.to_str())
                .map(|s| s.chars().all(|c| c.is_ascii_digit()))
                .unwrap_or(false);
            if !pid_ok {
                continue;
            }
            if let Ok(comm) = fs::read_to_string(path.join("comm")) {
                if let Some(id) = classify(comm.trim()) {
                    hits.insert(id);
                }
            }
        }

        // Always return the static suggestion list so the
        // .noActiveAgent state has something to render.
        Ok(AgentDetectorOut {
            running_count: hits.len() as u32,
            suggestions: known_suggestions(),
        })
    }
}

fn known_suggestions() -> Vec<AgentSuggestionOut> {
    [
        ("cursor", "Cursor"),
        ("claude-desktop", "Claude Desktop"),
        ("vscode", "VS Code"),
        ("zed", "Zed"),
    ]
    .iter()
    .map(|(id, label)| AgentSuggestionOut {
        id: id.to_string(),
        display_name: label.to_string(),
        bundle_identifier: None,
    })
    .collect()
}
