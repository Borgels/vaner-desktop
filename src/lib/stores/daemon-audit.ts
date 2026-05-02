// daemon-audit.ts — listens for the `daemon:strays` event the Rust
// side fires once at startup. Whatever PIDs come back are processes
// the desktop didn't spawn (foreign `vaner daemon / up / proxy / mcp`).
// Surfacing them once gives the user a way to reclaim ownership of
// the daemon lifecycle without dropping into a terminal.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { showToast } from "./toast.js";

export interface StrayProcess {
  pid: number;
  cmdline: string;
  kind: "daemon" | "up" | "proxy" | "mcp";
  path?: string;
}

const internal = writable<StrayProcess[]>([]);
export const daemonStrays: Readable<StrayProcess[]> = {
  subscribe: internal.subscribe,
};

let unlisten: UnlistenFn | null = null;

export async function bootstrapDaemonAudit(): Promise<void> {
  if (unlisten) return;
  unlisten = await listen<StrayProcess[]>("daemon:strays", (event) => {
    internal.set(event.payload ?? []);
  });
}

export async function disposeDaemonAudit(): Promise<void> {
  unlisten?.();
  unlisten = null;
}

/** Clear the in-memory list without touching the actual processes.
 *  Used by the banner's "Later" button. The strays will re-appear on
 *  next desktop launch since the audit runs at startup. */
export function dismissDaemonStrays(): void {
  internal.set([]);
}

/** Re-run the audit on demand (e.g. after the user clicks "Stop
 *  these" — confirms the kills landed). */
export async function refreshDaemonAudit(): Promise<void> {
  try {
    const strays = await invoke<StrayProcess[]>("audit_strays");
    internal.set(strays);
  } catch (err) {
    showToast(
      err instanceof Error ? err.message : `Could not audit daemons: ${err}`,
      "attention",
      4000,
    );
  }
}

export async function killStrays(pids: number[]): Promise<void> {
  if (pids.length === 0) return;
  try {
    const sent = await invoke<number>("kill_strays", { pids });
    showToast(
      sent === pids.length
        ? `Stopped ${sent} stray ${sent === 1 ? "process" : "processes"}.`
        : `Stopped ${sent} of ${pids.length} (some may need a stronger signal).`,
      "success",
      3500,
    );
    // Re-audit so the banner clears.
    await refreshDaemonAudit();
  } catch (err) {
    showToast(
      err instanceof Error ? err.message : `Could not stop processes: ${err}`,
      "attention",
      4000,
    );
  }
}
