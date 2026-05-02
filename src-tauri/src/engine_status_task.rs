//! Single-source poller for engine status.
//!
//! Pre-fix the popover and companion windows each kept their own
//! polling loop — Tauri webviews are isolated JS contexts, so a stale
//! stub in one window could disagree with a fresh probe in the other.
//! Result: companion shows "Running ✓", popover shows "ENGINE
//! UNAVAILABLE", same engine, same instant. Wrong at the architecture
//! level, not just the UI level.
//!
//! This module owns the cadence on the Rust side. One tokio task
//! shells `vaner status --json` every [`BASE_INTERVAL`] (faster
//! during a [`boost`] window), caches the latest [`EngineStatusOut`],
//! and emits an `engine:status` event the moment anything changes.
//! Every Tauri webview hydrates from the cache on mount and listens
//! to the event for updates — they get bit-identical state, every
//! time.
//!
//! `engine_status` (the Tauri command) just reads the cache. There
//! is no longer a per-window probe.

use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter};
use tokio::sync::RwLock;
use tokio::time::Instant;

use crate::engine::{EngineStatusOut, probe_engine_status};

const BASE_INTERVAL: Duration = Duration::from_secs(5);
const BOOST_INTERVAL: Duration = Duration::from_millis(500);

#[derive(Debug)]
pub struct EngineStatusCache {
    snapshot: RwLock<EngineStatusOut>,
    boost_until: RwLock<Option<Instant>>,
    notify: tokio::sync::Notify,
}

impl Default for EngineStatusCache {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineStatusCache {
    pub fn new() -> Self {
        Self {
            snapshot: RwLock::new(EngineStatusOut {
                reachable: false,
                cli_missing: false,
                files_watched: 0,
                sources_count: 0,
                uptime_minutes: 0,
                indexing_kind: "idle".to_string(),
                detail: None,
            }),
            boost_until: RwLock::new(None),
            notify: tokio::sync::Notify::new(),
        }
    }

    pub async fn snapshot(&self) -> EngineStatusOut {
        self.snapshot.read().await.clone()
    }

    /// Boost the poller to [`BOOST_INTERVAL`] for `dur`. Idempotent:
    /// overlapping boost calls extend the window rather than queue.
    pub async fn boost(&self, dur: Duration) {
        let new_until = Instant::now() + dur;
        let mut guard = self.boost_until.write().await;
        match *guard {
            Some(existing) if existing > new_until => {}
            _ => *guard = Some(new_until),
        }
        // Wake the poll loop immediately so the user doesn't wait
        // 5s for the first boosted tick.
        self.notify.notify_one();
    }

    async fn current_interval(&self) -> Duration {
        match *self.boost_until.read().await {
            Some(until) if Instant::now() < until => BOOST_INTERVAL,
            _ => BASE_INTERVAL,
        }
    }
}

/// Spawn the background poll loop. Call once from `lib.rs` setup().
pub fn spawn(app: AppHandle, cache: Arc<EngineStatusCache>) {
    tauri::async_runtime::spawn(async move {
        loop {
            let next = probe_engine_status().await;
            let changed = {
                let mut guard = cache.snapshot.write().await;
                let differs = !same(&guard, &next);
                *guard = next.clone();
                differs
            };
            if changed {
                let _ = app.emit("engine:status", &next);
            }
            let interval = cache.current_interval().await;
            // Race the timer against `boost()`'s notify so a click on
            // "Restart engine" gets a snappy first probe.
            tokio::select! {
                _ = tokio::time::sleep(interval) => {}
                _ = cache.notify.notified() => {}
            }
        }
    });
}

fn same(a: &EngineStatusOut, b: &EngineStatusOut) -> bool {
    a.reachable == b.reachable
        && a.cli_missing == b.cli_missing
        && a.files_watched == b.files_watched
        && a.sources_count == b.sources_count
        && a.uptime_minutes == b.uptime_minutes
        && a.indexing_kind == b.indexing_kind
        && a.detail == b.detail
}

/// Tauri command — request a fast-poll burst. Called from `Restart
/// engine` / `Start engine` so the popover flips out of `.error`
/// within a fraction of a second of the cockpit answering.
#[tauri::command]
pub async fn engine_status_boost(
    duration_ms: u64,
    cache: tauri::State<'_, Arc<EngineStatusCache>>,
) -> Result<(), String> {
    let clamped = duration_ms.clamp(500, 30_000);
    cache.boost(Duration::from_millis(clamped)).await;
    Ok(())
}
