// 0.8.5 WS12-D — MCP clients store.
//
// Wraps the new Tauri commands `clients_detect`, `clients_install`,
// `clients_install_all`, `clients_uninstall`, `clients_doctor` (each
// shells out to the `vaner clients` CLI on the Rust side). The CLI is
// the single source of truth for per-client config paths, atomic +
// backup-rotated writes, and idempotent merges.

import { invoke } from "@tauri-apps/api/core";
import { derived, writable, type Readable } from "svelte/store";

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

export type WriteLayerResult = {
  layer: string;
  applicable: boolean;
  /** Broader than pre-Phase-C: "added" / "updated" / "skipped" /
   *  "failed" / "not-applicable". We don't enum-narrow it because
   *  the CLI may add new layers (e.g. "plugin"). */
  action: string;
  path: string | null;
  error: string | null;
};

export type WriteResult = {
  client_id: string;
  label: string;
  detected: boolean;
  /** "ready" / "wired-mcp-only" / "partial" / "missing" / "not-detected" */
  overall: string;
  layers: WriteLayerResult[];
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

/** Reducer-input projection of the clients store. The reducer cares
 *  only about (a) whether the detector has run, and (b) how many
 *  clients have Vaner wired in — not the full per-client metadata.
 *  `total === 0` means the detector hasn't completed yet (or failed),
 *  which the reducer treats as "don't gate, fall through to engine
 *  state". */
export const clientDetectStatus = derived<typeof clients, {
  total: number;
  wiredCount: number;
  wiredLabels: string[];
}>(internal, ($state) => {
  if (!$state.hasInitialScan) {
    return { total: 0, wiredCount: 0, wiredLabels: [] };
  }
  const wired = $state.clients.filter((c) => c.configured);
  return {
    total: $state.clients.length,
    wiredCount: wired.length,
    wiredLabels: wired.map((c) => c.label),
  };
});

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
