// Live engine status — populated by polling the Rust `engine_status`
// command. The reducer's stub default (reachable=true / 0 sources)
// keeps the popover landing on .installedNotConnected at startup;
// the first poll replaces it with the real shape.

import { invoke } from "@tauri-apps/api/core";
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

let pollHandle: ReturnType<typeof setInterval> | null = null;

/** Begin polling `engine_status` from Rust every `intervalMs`. Idempotent.
 *  `sources_count` is overlaid from setup_status by a separate caller —
 *  this poll only learns reachability + indexing kind. */
export function startEngineStatusPolling(intervalMs = 5000): void {
  if (pollHandle != null) return;
  const tick = async () => {
    try {
      const out = await invoke<EngineStatusOut>("engine_status");
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
    } catch {
      // Defensive: invoke itself failed (Tauri runtime issue, not a
      // CLI-missing case — the Rust side returns Ok with cli_missing
      // for that). Flag unreachable but don't claim cliMissing.
      engineStatus.update((prev) => ({ ...prev, reachable: false }));
    }
  };
  void tick();
  pollHandle = setInterval(tick, intervalMs);
}

export function stopEngineStatusPolling(): void {
  if (pollHandle != null) {
    clearInterval(pollHandle);
    pollHandle = null;
  }
}

/** Overlay `sourcesCount` from setup_status. Called from the layout
 *  after loadStatus resolves — without this the reducer can't tell
 *  the difference between .installedNotConnected and .watching. */
export function setSourcesCount(count: number): void {
  engineStatus.update((prev) => ({ ...prev, sourcesCount: count }));
}
