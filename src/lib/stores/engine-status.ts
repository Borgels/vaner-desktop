// Live engine status — hydrated by listening to `engine:status`
// events emitted from the Rust-side single-source poller in
// `src-tauri/src/engine_status_task.rs`. The cadence (and probe
// itself) live there so every webview sees the same boolean at the
// same time. Per-window polling was the source of popover/companion
// disagreement before this refactor.
//
// Each layout calls `startEngineStatusListener()` on mount; the call
// is idempotent across windows and across re-mounts inside one
// window. The Tauri event bus broadcasts to all webviews, so the
// popover and the companion's Engine pane react to the same payload
// at the same moment.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable, type Writable } from "svelte/store";
import type { EngineStatus } from "$lib/state/types.js";

const stub: EngineStatus = {
  reachable: true,
  cliMissing: false,
  filesWatched: 0,
  sourcesCount: 0,
  uptimeMinutes: 0,
  lastCycleSecondsAgo: null,
  cycleIntervalSeconds: 60,
  indexing: { kind: "idle" },
};

interface EngineStatusOut {
  reachable: boolean;
  cli_missing: boolean;
  files_watched: number;
  sources_count: number;
  uptime_minutes: number;
  indexing_kind: string;
  detail: string | null;
}

export const engineStatus: Writable<EngineStatus> = writable(stub);

export function setEngineStatus(s: EngineStatus): void {
  engineStatus.set(s);
}

function applyOut(out: EngineStatusOut): void {
  engineStatus.update((prev) => ({
    ...prev,
    reachable: out.reachable,
    cliMissing: out.cli_missing,
    filesWatched: out.files_watched,
    // Keep prev.sourcesCount until the setup-status overlay fires
    // (we don't want to flap the reducer between
    // .installedNotConnected and .watching).
    uptimeMinutes: out.uptime_minutes || prev.uptimeMinutes,
    indexing: out.indexing_kind === "learning"
      ? { kind: "learning", currentlyReading: [], etaMinutes: null }
      : { kind: "idle" },
  }));
}

let unlisten: UnlistenFn | null = null;
let started = false;

/** Subscribe to the Rust-side `engine:status` event stream. Idempotent
 *  across calls and across windows. The first call also pulls the
 *  current cached snapshot so the UI doesn't flash the stub. */
export async function startEngineStatusListener(): Promise<void> {
  if (started) return;
  started = true;

  // Hydrate from the current cache so first paint is correct.
  try {
    const snapshot = await invoke<EngineStatusOut>("engine_status");
    applyOut(snapshot);
  } catch {
    engineStatus.update((prev) => ({ ...prev, reachable: false }));
  }

  unlisten = await listen<EngineStatusOut>("engine:status", (e) => {
    applyOut(e.payload);
  });
}

export function stopEngineStatusListener(): void {
  unlisten?.();
  unlisten = null;
  started = false;
}

// Backwards-compatible names — earlier code called these. They now
// just route through the listener so older call sites keep working.
export const startEngineStatusPolling = startEngineStatusListener;
export const stopEngineStatusPolling = stopEngineStatusListener;

/** Boost the Rust-side poller to 500ms for `durationMs` (default
 *  10s). Called after a bring-up / restart so the popover flips out
 *  of `.error` within a fraction of a second of the cockpit
 *  answering rather than waiting up to a full base interval. */
export async function boostEngineStatusPolling(durationMs = 10_000): Promise<void> {
  try {
    await invoke("engine_status_boost", { durationMs });
  } catch {
    // Best-effort: if the Rust side is still booting the boost
    // command may not be registered yet. Next tick of the base
    // interval will land soon enough.
  }
}

/** Overlay `sourcesCount` from setup_status. Called from the layout
 *  after loadStatus resolves — without this the reducer can't tell
 *  the difference between .installedNotConnected and .watching. */
export function setSourcesCount(count: number): void {
  engineStatus.update((prev) => ({ ...prev, sourcesCount: count }));
}
