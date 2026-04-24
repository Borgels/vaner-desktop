import { writable, type Readable } from "svelte/store";
import type { UnlistenFn } from "@tauri-apps/api/event";
import { listen } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import type { PredictedPrompt } from "$lib/contract/types";

// The Rust backend emits `predictions:snapshot` whenever the SSE
// stream delivers a new frame. This store mirrors the latest
// snapshot to every subscribed Svelte component.
const { subscribe, set }: { subscribe: Readable<PredictedPrompt[]>["subscribe"]; set: (v: PredictedPrompt[]) => void } =
  writable<PredictedPrompt[]>([]);

let unlisten: UnlistenFn | null = null;
let started = false;

/** Subscribe to live prediction snapshots. Safe to call multiple times. */
export async function startPredictionStream(): Promise<void> {
  if (started) return;
  started = true;

  // Pull a first snapshot so the UI has something before SSE ticks.
  try {
    const initial = await invoke<PredictedPrompt[]>("active_predictions");
    set(initial);
  } catch (err) {
    // Engine unreachable on startup is not fatal — leave empty and let
    // the SSE backoff retry loop recover.
    console.warn("[vaner] initial active_predictions failed:", err);
  }

  unlisten = await listen<PredictedPrompt[]>("predictions:snapshot", (event) => {
    set(event.payload);
  });
}

/** Tear the subscription down — tests / teardown only. */
export async function stopPredictionStream(): Promise<void> {
  if (unlisten) {
    unlisten();
    unlisten = null;
  }
  started = false;
}

export const predictions = { subscribe };
