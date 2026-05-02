//! Stray-daemon detection.
//!
//! The desktop is supposed to own the daemon's lifecycle. If the user
//! has another `vaner daemon start --once` loop, a manual `vaner up`,
//! a `vaner proxy`, or a forgotten `vaner mcp` running in the
//! background, those compete for system resources and (in the
//! daemon-start case) drive a model loop with no one watching the
//! output. Surface them to the user once and let them stop.
//!
//! We define a "stray" as a `vaner` invocation matching one of:
//!   - `vaner daemon …`   (the engine cycler)
//!   - `vaner up …`       (a manual cockpit/daemon supervisor)
//!   - `vaner proxy …`    (the OpenAI-compatible proxy)
//!   - `vaner mcp …`      (an MCP server outside an active editor)
//!
//! We exclude:
//!   - Our own PID and any process whose parent is our PID.
//!   - Anything whose cmdline points at the same `--path` the desktop
//!     uses (i.e. the daemon *we* spawned via bring_up).
//!   - vaner-desktop itself (the GUI binary, distinct from the CLI).
//!
//! Tauri commands:
//!   - [`audit_strays`]   — return the current stray list.
//!   - [`kill_strays`]    — SIGTERM the listed PIDs.

use std::path::Path;

use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct StrayProcess {
    pub pid: i32,
    /// Full command line, joined with spaces. NUL separators in
    /// `/proc/PID/cmdline` get replaced for display.
    pub cmdline: String,
    /// First arg-after-`vaner` token — `daemon` / `up` / `proxy` /
    /// `mcp` — so the UI can group "engine cyclers" vs "MCP shells".
    pub kind: String,
    /// Resolved `--path` argument when present, for display.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

fn read_cmdline(pid: i32) -> Option<Vec<String>> {
    let bytes = std::fs::read(format!("/proc/{pid}/cmdline")).ok()?;
    if bytes.is_empty() {
        return None;
    }
    Some(
        bytes
            .split(|b| *b == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf8_lossy(s).into_owned())
            .collect(),
    )
}

fn read_ppid(pid: i32) -> Option<i32> {
    // /proc/PID/stat: `pid (comm) state ppid …`. `comm` may contain
    // spaces or parens, so use the last `)` as the boundary.
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    let close = stat.rfind(')')?;
    let rest = &stat[close + 1..];
    let mut tokens = rest.split_whitespace();
    tokens.next()?; // state
    tokens.next()?.parse().ok() // ppid
}

fn classify(args: &[String]) -> Option<(String, Option<String>)> {
    // First arg is the binary path (or `vaner` itself when run via
    // shebang). We accept either when the first non-binary token is
    // a known subcommand.
    let mut iter = args.iter().peekable();
    let first = iter.next()?;
    let binary_looks_like_vaner = first.ends_with("/vaner")
        || first == "vaner"
        || (!first.ends_with("/vaner-desktop")
            && first.split('/').next_back().map(|s| s.starts_with("vaner")) == Some(true));
    // The python wrapper uses `python /path/to/vaner …` — peek one
    // token deeper if the first looks like `python` and the next
    // ends with `/vaner`.
    let after = if !binary_looks_like_vaner
        && first.split('/').next_back() == Some("python")
        && let Some(next) = iter.peek()
        && (next.ends_with("/vaner") || *next == "vaner")
    {
        iter.next()?;
        iter.next()?
    } else if binary_looks_like_vaner {
        iter.next()?
    } else {
        return None;
    };

    let kind = match after.as_str() {
        "daemon" | "up" | "proxy" | "mcp" => after.clone(),
        _ => return None,
    };

    // Walk remaining args for `--path <X>`.
    let mut path = None;
    let mut tokens: Vec<String> = iter.cloned().collect();
    while !tokens.is_empty() {
        let t = tokens.remove(0);
        if t == "--path" && !tokens.is_empty() {
            path = Some(tokens.remove(0));
            break;
        }
        if let Some(rest) = t.strip_prefix("--path=") {
            path = Some(rest.to_string());
            break;
        }
    }
    Some((kind, path))
}

fn is_descendant_of(mut pid: i32, ancestor: i32) -> bool {
    for _ in 0..16 {
        // bounded ancestor walk
        match read_ppid(pid) {
            Some(ppid) if ppid == ancestor => return true,
            Some(0) | None => return false,
            Some(ppid) if ppid == pid => return false,
            Some(ppid) => pid = ppid,
        }
    }
    false
}

/// Enumerate stray vaner processes (criteria above). Best-effort: any
/// `/proc` read failure for a single PID just skips that entry.
pub fn find_strays() -> Vec<StrayProcess> {
    let our_pid = std::process::id() as i32;
    let workspace = crate::workspace::resolve();
    let workspace_str = workspace.as_ref().map(|p| p.to_string_lossy().into_owned());

    let proc_dir = match std::fs::read_dir("/proc") {
        Ok(d) => d,
        Err(_) => return Vec::new(),
    };

    let mut out: Vec<StrayProcess> = Vec::new();
    for entry in proc_dir.flatten() {
        let Ok(name) = entry.file_name().into_string() else {
            continue;
        };
        let Ok(pid) = name.parse::<i32>() else {
            continue;
        };
        if pid == our_pid {
            continue;
        }
        let Some(args) = read_cmdline(pid) else {
            continue;
        };
        // Exclude vaner-desktop itself (the GUI binary, separate from
        // the Python CLI).
        if args
            .first()
            .map(|s| s.split('/').next_back() == Some("vaner-desktop"))
            .unwrap_or(false)
        {
            continue;
        }
        let Some((kind, path)) = classify(&args) else {
            continue;
        };
        // Skip our own descendants (the daemon we spawned via
        // bring_up should not show up as a stray).
        if is_descendant_of(pid, our_pid) {
            continue;
        }
        // Skip processes targeting our active workspace — those are
        // legitimate (probably ours from a prior session that
        // out-lived us, but the user already opted in to that
        // workspace, so killing them silently would be surprising).
        if let (Some(p), Some(w)) = (&path, &workspace_str) {
            if Path::new(p) == Path::new(w) {
                continue;
            }
        }
        let cmdline = args.join(" ");
        out.push(StrayProcess {
            pid,
            cmdline,
            kind,
            path,
        });
    }
    out
}

#[tauri::command]
pub fn audit_strays() -> Vec<StrayProcess> {
    find_strays()
}

#[tauri::command]
pub fn kill_strays(pids: Vec<i32>) -> Result<usize, String> {
    if pids.is_empty() {
        return Ok(0);
    }
    // SIGTERM via libc::kill on Unix; Windows doesn't have signals
    // (and the audit logic above is /proc-only anyway, so this code
    // path can't fire on Windows in practice — but cargo check
    // still wants the symbol resolvable).
    #[cfg(unix)]
    {
        let mut sent = 0usize;
        for pid in pids {
            // SAFETY: kill(2) is signal-safe; we own no shared mutable
            // state that this could disturb.
            let rc = unsafe { libc::kill(pid, libc::SIGTERM) };
            if rc == 0 {
                sent += 1;
            }
        }
        Ok(sent)
    }
    #[cfg(not(unix))]
    {
        let _ = pids;
        Err("kill_strays is unix-only".to_string())
    }
}
