// Live engine status — populated by the Rust `engine_status` command in WS8.
// Stub for now: assume reachable, no indexing, modest file count. The reducer
// falls through to `.watching` until real data lands.

import { writable, type Writable } from "svelte/store";
import type { EngineStatus } from "$lib/state/types.js";

const stub: EngineStatus = {
  reachable: true,
  filesWatched: 0,
  sourcesCount: 0,
  uptimeMinutes: 0,
  lastCycleSecondsAgo: null,
  cycleIntervalSeconds: 60,
  indexing: { kind: "idle" },
};

export const engineStatus: Writable<EngineStatus> = writable(stub);

/** Replace the whole status snapshot. Called from WS8's tauri command bridge. */
export function setEngineStatus(s: EngineStatus): void {
  engineStatus.set(s);
}
