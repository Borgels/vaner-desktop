// workspace.ts — Svelte store mirroring the Rust-side workspace
// resolution (workspace_get / workspace_set / workspace_pick). The
// store is the single source of truth the reducer consumes via
// `workspaceMissing`; everything else (Preferences pane, onboarding
// step, the `.needsWorkspace` popover) reads/writes through here.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

const internal = writable<string | null>(null);

export const workspacePath: Readable<string | null> = { subscribe: internal.subscribe };

let loaded = false;

/** Hydrate the store from Rust-side state.json. Called from the root
 *  layout's onMount. Idempotent — repeated calls are no-ops after the
 *  first successful load. */
export async function loadWorkspace(): Promise<void> {
  if (loaded) return;
  try {
    const path = await invoke<string | null>("workspace_get");
    internal.set(path ?? null);
    loaded = true;
  } catch {
    // Tauri call failed — leave the store as null. The reducer will
    // surface .needsWorkspace, which gives the user a way out.
    internal.set(null);
  }
}

/** Open the native folder picker. Returns the chosen path on success,
 *  null on cancel. Persists in state.json on the Rust side. */
export async function pickWorkspace(): Promise<string | null> {
  const result = await invoke<string | null>("workspace_pick");
  internal.set(result ?? null);
  return result ?? null;
}

/** Set the workspace from a known-good absolute path (e.g. when the
 *  user types one into Preferences). Throws if Rust-side validation
 *  rejects it (path missing, not a directory, not absolute). */
export async function setWorkspace(path: string): Promise<string> {
  const stored = await invoke<string>("workspace_set", { path });
  internal.set(stored);
  return stored;
}
