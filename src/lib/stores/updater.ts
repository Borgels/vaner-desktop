import { writable } from "svelte/store";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { showToast } from "./toast.js";

export interface UpdateInfo {
  version: string;
  currentVersion: string;
  notes?: string | null;
}

/** `null` until a newer release is found. */
export const availableUpdate = writable<UpdateInfo | null>(null);
/** 0–1 when an install is in progress, `null` otherwise. */
export const updateProgress = writable<number | null>(null);

let unlisteners: UnlistenFn[] = [];
let booted = false;

export async function bootstrapUpdaterListeners(): Promise<void> {
  if (booted) return;
  booted = true;

  unlisteners.push(
    await listen<{ version: string; current_version: string; release_notes: string | null }>(
      "update:available",
      ({ payload }) => {
        availableUpdate.set({
          version: payload.version,
          currentVersion: payload.current_version,
          notes: payload.release_notes,
        });
      },
    ),
  );

  unlisteners.push(
    await listen<number>("update:progress", ({ payload }) => {
      updateProgress.set(payload);
    }),
  );

  unlisteners.push(
    await listen<void>("update:ready-to-restart", () => {
      updateProgress.set(1);
      showToast(
        "Update installed. Restart Vaner to finish.",
        "success",
        8000,
      );
    }),
  );
}

export async function installUpdate(): Promise<void> {
  updateProgress.set(0);
  try {
    await invoke("install_update");
  } catch (err) {
    updateProgress.set(null);
    const msg = typeof err === "string" ? err : "Update install failed.";
    showToast(msg, "attention", 5000);
  }
}
