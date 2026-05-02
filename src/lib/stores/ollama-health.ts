// ollama-health.ts — mirrors `src-tauri/src/ollama_health_task.rs`.
// One Rust-side poller probes Ollama's presence + reachability and
// emits `ollama:health` events; both windows hydrate from the cache
// and listen to the event so they always agree on whether Ollama is
// available.

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { writable, type Readable } from "svelte/store";

export interface OllamaHealth {
  /** True when an `ollama` binary is on PATH or in a canonical
   *  install dir. False → popover routes to `.ollamaMissing`. */
  installed: boolean;
  /** True when the daemon answered `127.0.0.1:11434/api/tags`.
   *  Implies `installed`. False with `installed=true` means "Ollama
   *  is installed but not running" — the popover surfaces a
   *  different CTA ("Start Ollama") for that. */
  running: boolean;
  detail: string;
}

const stub: OllamaHealth = {
  // Optimistic stub — same trick as engine-status. The first event
  // overwrites it within ~150ms after window mount, so we'd rather
  // not flash the .ollamaMissing card to users who do have Ollama.
  installed: true,
  running: true,
  detail: "",
};

const internal = writable<OllamaHealth>(stub);

export const ollamaHealth: Readable<OllamaHealth> = {
  subscribe: internal.subscribe,
};

let unlisten: UnlistenFn | null = null;
let started = false;

export async function startOllamaHealthListener(): Promise<void> {
  if (started) return;
  started = true;
  try {
    const snapshot = await invoke<OllamaHealth>("ollama_health");
    internal.set(snapshot);
  } catch {
    // Tauri runtime not ready yet — leave the stub in place; the
    // listener below will catch the first event when the task starts.
  }
  unlisten = await listen<OllamaHealth>("ollama:health", (e) => {
    internal.set(e.payload);
  });
}

export function stopOllamaHealthListener(): void {
  unlisten?.();
  unlisten = null;
  started = false;
}

/** Launch the Ollama installer in a terminal. Returns when the
 *  terminal has spawned (the install itself runs interactively); the
 *  Rust side pokes the cache when the spawn completes so the popover
 *  flips out of `.ollamaMissing` on the next probe. */
export async function installOllama(): Promise<void> {
  await invoke("install_ollama");
}
