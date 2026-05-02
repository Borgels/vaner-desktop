// ollama.ts — mirrors the Rust `ollama_*` Tauri commands. The
// Models card binds to these stores so the UI can list installed
// models, pull new ones (with streaming progress), and remove them
// without dropping into a terminal.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface InstalledModel {
  name: string;
  size_bytes: number;
  size_display: string;
  modified_at?: string;
}

export interface OllamaListResult {
  available: boolean;
  models: InstalledModel[];
  detail: string;
}

export interface PullProgress {
  model: string;
  status: string;
  fraction: number | null;
  completed_bytes: number | null;
  total_bytes: number | null;
}

export interface PullDone {
  model: string;
  success: boolean;
  detail: string;
}

const ollamaStateInternal = writable<OllamaListResult>({
  available: false,
  models: [],
  detail: "Loading…",
});

export const ollamaState: Readable<OllamaListResult> = {
  subscribe: ollamaStateInternal.subscribe,
};

const activePullInternal = writable<PullProgress | null>(null);
export const activePull: Readable<PullProgress | null> = {
  subscribe: activePullInternal.subscribe,
};

let unlistenProgress: UnlistenFn | null = null;
let unlistenDone: UnlistenFn | null = null;
let bootstrapped = false;

/** Register Tauri event listeners for pull-progress / pull-done.
 *  Idempotent — repeated calls are no-ops. */
export async function bootstrapOllamaListeners(): Promise<void> {
  if (bootstrapped) return;
  bootstrapped = true;
  unlistenProgress = await listen<PullProgress>("ollama:pull-progress", (event) => {
    activePullInternal.set(event.payload);
  });
  unlistenDone = await listen<PullDone>("ollama:pull-done", (_event) => {
    activePullInternal.set(null);
    // Re-list so the just-pulled model appears in the installed list,
    // and the just-removed model disappears.
    void refreshOllama();
  });
}

export async function disposeOllamaListeners(): Promise<void> {
  unlistenProgress?.();
  unlistenDone?.();
  unlistenProgress = null;
  unlistenDone = null;
  bootstrapped = false;
}

export async function refreshOllama(): Promise<OllamaListResult> {
  try {
    const result = await invoke<OllamaListResult>("ollama_list");
    ollamaStateInternal.set(result);
    return result;
  } catch (err) {
    const detail = err instanceof Error ? err.message : `${err}`;
    const fallback: OllamaListResult = { available: false, models: [], detail };
    ollamaStateInternal.set(fallback);
    return fallback;
  }
}

/** Kick off a pull. Returns immediately — the UI reacts to
 *  `ollama:pull-progress` / `ollama:pull-done`. Throws on bad input
 *  (empty name) so the form can flag the field. */
export async function pullModel(name: string): Promise<void> {
  await invoke<void>("ollama_pull", { name });
}

export async function cancelPull(): Promise<void> {
  await invoke<void>("ollama_cancel_pull");
  activePullInternal.set(null);
}

export async function removeModel(name: string): Promise<void> {
  await invoke<void>("ollama_remove", { name });
  await refreshOllama();
}

/** Set this model as Vaner's `backend.model`. Reuses the existing
 *  `set_local_model` Tauri command (in diagnostics.rs) so cloud
 *  preset switching and Ollama model selection share one path. */
export async function useModel(name: string): Promise<void> {
  await invoke<string>("set_local_model", { modelId: name });
}
