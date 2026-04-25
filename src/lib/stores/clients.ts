// 0.8.5 WS12-D — MCP clients store.
//
// Wraps the new Tauri commands `clients_detect`, `clients_install`,
// `clients_install_all`, `clients_uninstall`, `clients_doctor` (each
// shells out to the `vaner clients` CLI on the Rust side). The CLI is
// the single source of truth for per-client config paths, atomic +
// backup-rotated writes, and idempotent merges.

import { invoke } from "@tauri-apps/api/core";
import { writable, type Readable } from "svelte/store";

// Wire types matching the Rust serde structs in src-tauri/src/clients.rs.
export type DetectedClient = {
  id: string;
  label: string;
  kind: string;
  status: "configured" | "installed" | "missing";
  detected: boolean;
  configured: boolean;
  config_path: string | null;
  detail: string;
};

export type WriteResult = {
  client_id: string;
  path: string | null;
  action: "added" | "updated" | "skipped" | "failed";
  backup: string | null;
  error: string | null;
  manual_snippet: string | null;
};

export type ClientDrift = {
  client_id: string;
  label: string;
  config_path: string | null;
  drift: boolean;
  current_in_config: string | null;
  expected: string;
  detail: string;
};

export type DoctorReport = {
  drift: ClientDrift[];
  drift_count: number;
  fix_command: string;
};

export type ClientsState = {
  clients: DetectedClient[];
  doctor: DoctorReport | null;
  isScanning: boolean;
  hasInitialScan: boolean;
  lastError: string | null;
  lastResults: WriteResult[];
};

const initial: ClientsState = {
  clients: [],
  doctor: null,
  isScanning: false,
  hasInitialScan: false,
  lastError: null,
  lastResults: [],
};

const internal = writable<ClientsState>(initial);

/** Repo root the user is currently attached to. The desktop app
 *  doesn't yet propagate the attached repo to the frontend, so default
 *  to `$HOME` (the `vaner clients` CLI tolerates a missing repo and
 *  emits a "no .vaner/config.toml" detail without erroring). */
function defaultRepoRoot(): string {
  return ""; // Rust resolves with current dir; we let the user override later.
}

async function patch(updater: (state: ClientsState) => Partial<ClientsState>): Promise<void> {
  internal.update((state) => ({ ...state, ...updater(state) }));
}

export const clients: Readable<ClientsState> = { subscribe: internal.subscribe };

export async function rescan(repoRoot: string = defaultRepoRoot()): Promise<void> {
  await patch(() => ({ isScanning: true }));
  try {
    const detectedRoot = repoRoot || defaultRepoRoot();
    const detected = await invoke<DetectedClient[]>("clients_detect", { repoRoot: detectedRoot });
    let doctor: DoctorReport | null = null;
    try {
      doctor = await invoke<DoctorReport>("clients_doctor", { repoRoot: detectedRoot });
    } catch (_err) {
      // Drift report is opportunistic; missing config or daemon isn't fatal here.
      doctor = null;
    }
    await patch(() => ({
      clients: detected,
      doctor,
      isScanning: false,
      hasInitialScan: true,
      lastError: null,
    }));
  } catch (err) {
    await patch(() => ({
      isScanning: false,
      hasInitialScan: true,
      lastError: typeof err === "string" ? err : String(err),
    }));
  }
}

export async function install(clientId: string, repoRoot = "", force = false): Promise<WriteResult[]> {
  try {
    const results = await invoke<WriteResult[]>("clients_install", {
      repoRoot,
      clientId,
      force,
    });
    await patch(() => ({ lastResults: results, lastError: null }));
    await rescan(repoRoot);
    return results;
  } catch (err) {
    await patch(() => ({ lastError: typeof err === "string" ? err : String(err) }));
    return [];
  }
}

export async function installAll(repoRoot = "", force = false): Promise<WriteResult[]> {
  try {
    const results = await invoke<WriteResult[]>("clients_install_all", { repoRoot, force });
    await patch(() => ({ lastResults: results, lastError: null }));
    await rescan(repoRoot);
    return results;
  } catch (err) {
    await patch(() => ({ lastError: typeof err === "string" ? err : String(err) }));
    return [];
  }
}

export async function uninstall(clientId: string, repoRoot = ""): Promise<WriteResult[]> {
  try {
    const results = await invoke<WriteResult[]>("clients_uninstall", {
      repoRoot,
      clientId,
    });
    await patch(() => ({ lastResults: results, lastError: null }));
    await rescan(repoRoot);
    return results;
  } catch (err) {
    await patch(() => ({ lastError: typeof err === "string" ? err : String(err) }));
    return [];
  }
}
