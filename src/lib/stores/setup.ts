// 0.8.6 WS-DESK-LINUX — Simple-Mode setup store.
//
// Wraps the new Tauri commands `setup_questions`, `setup_recommend`,
// `setup_apply`, `setup_status`, `policy_show`, `policy_refresh`,
// `hardware_profile`, `deep_run_defaults`. Each Tauri command shells
// out to `vaner setup ... --json` (matching the 0.8.5 WS12-D
// `clients_*` pattern); the CLI is the single source of truth for
// bundle selection, atomic config writes, and cloud-widening guards.
//
// The store is intentionally thin: it surfaces loading flags + the
// last error + the most recent payloads, and lets pages re-call any
// of the methods to refresh. Components subscribe to `setup` for the
// snapshot or call methods directly for one-shot reads.

import { invoke } from "@tauri-apps/api/core";
import { writable, type Readable } from "svelte/store";

import type {
  AppliedPolicy,
  DeepRunDefaults,
  HardwareProfile,
  SelectionResult,
  SetupAnswers,
  SetupQuestion,
  SetupStatus,
  VanerPolicyBundle,
} from "$lib/contract/setup-types.js";

export type SetupMode = "simple" | "advanced";

/** Local-storage key for the Engine-tab Simple/Advanced toggle. The
 *  engine `setup.mode` field is the canonical record once the user
 *  hits Apply; this key just remembers the *UI* preference between
 *  loads while the user is still editing. */
const SETUP_MODE_LS_KEY = "vaner.pref.setupMode";

export type SetupState = {
  status: SetupStatus | null;
  hardware: HardwareProfile | null;
  bundle: VanerPolicyBundle | null;
  lastSelection: SelectionResult | null;
  lastApply: AppliedPolicy | null;
  deepRunDefaults: DeepRunDefaults | null;
  isLoading: boolean;
  hasInitialLoad: boolean;
  lastError: string | null;
};

const initial: SetupState = {
  status: null,
  hardware: null,
  bundle: null,
  lastSelection: null,
  lastApply: null,
  deepRunDefaults: null,
  isLoading: false,
  hasInitialLoad: false,
  lastError: null,
};

const internal = writable<SetupState>(initial);

async function patch(updater: (state: SetupState) => Partial<SetupState>): Promise<void> {
  internal.update((state) => ({ ...state, ...updater(state) }));
}

export const setup: Readable<SetupState> = { subscribe: internal.subscribe };

// ---------------------------------------------------------------------
// Setup-mode persistence (Engine tab Simple/Advanced toggle)
// ---------------------------------------------------------------------

function readSetupMode(): SetupMode {
  if (typeof window === "undefined") return "simple";
  try {
    const stored = window.localStorage.getItem(SETUP_MODE_LS_KEY);
    return stored === "advanced" ? "advanced" : "simple";
  } catch {
    return "simple";
  }
}

export const setupMode = writable<SetupMode>(readSetupMode());

if (typeof window !== "undefined") {
  setupMode.subscribe((value) => {
    try {
      window.localStorage.setItem(SETUP_MODE_LS_KEY, value);
    } catch {
      // localStorage may be unavailable (private mode); silently skip.
    }
  });
}

// ---------------------------------------------------------------------
// Tauri-backed methods
// ---------------------------------------------------------------------

/** Load the question schema. Hand-mirrored on the Rust side until the
 *  daemon ships `GET /setup/questions`. */
export async function loadQuestions(): Promise<SetupQuestion[]> {
  try {
    return await invoke<SetupQuestion[]>("setup_questions");
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return [];
  }
}

/** `vaner setup show --json` — current setup + policy + hardware
 *  state. The wizard checks `status.setup.completed_at` to decide
 *  whether to fire the first-run flow. */
export async function loadStatus(): Promise<SetupStatus | null> {
  await patch(() => ({ isLoading: true }));
  try {
    const status = await invoke<SetupStatus>("setup_status");
    await patch(() => ({
      status,
      hardware: status.hardware,
      bundle: status.bundle,
      isLoading: false,
      hasInitialLoad: true,
      lastError: null,
    }));
    return status;
  } catch (err) {
    await patch(() => ({
      isLoading: false,
      hasInitialLoad: true,
      lastError: stringErr(err),
    }));
    return null;
  }
}

/** `vaner setup recommend --json` — pure recommendation; never writes. */
export async function recommend(answers: SetupAnswers): Promise<SelectionResult | null> {
  try {
    const selection = await invoke<SelectionResult>("setup_recommend", { answers });
    await patch(() => ({ lastSelection: selection, lastError: null }));
    return selection;
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return null;
  }
}

export type ApplyArgs = {
  answers?: SetupAnswers;
  bundle_id?: string;
  confirm_cloud_widening?: boolean;
};

/** `vaner setup apply --json` — persist + return AppliedPolicy.
 *  When `widens_cloud_posture` is true and the user has not
 *  confirmed, the engine declines to write (`written=false`); the
 *  caller surfaces the warning and re-calls with
 *  `confirm_cloud_widening=true`. */
export async function apply(args: ApplyArgs): Promise<AppliedPolicy | null> {
  try {
    const result = await invoke<AppliedPolicy>("setup_apply", { payload: args });
    await patch(() => ({ lastApply: result, lastError: null }));
    if (result.written) {
      // Refresh status so the Engine / Telemetry tabs see the new bundle.
      await loadStatus();
    }
    return result;
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return null;
  }
}

/** `vaner setup hardware --json` — fresh probe. Cheap enough to call
 *  on demand from the Telemetry tab. */
export async function loadHardware(): Promise<HardwareProfile | null> {
  try {
    const hardware = await invoke<HardwareProfile>("hardware_profile");
    await patch(() => ({ hardware, lastError: null }));
    return hardware;
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return null;
  }
}

/** Re-read the bundle currently selected in `[policy]`. Backed by
 *  `setup show --json` because no separate `vaner policy show`
 *  command exists yet. */
export async function loadPolicy(): Promise<VanerPolicyBundle | null> {
  try {
    const bundle = await invoke<VanerPolicyBundle | null>("policy_show");
    await patch(() => ({ bundle, lastError: null }));
    return bundle;
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return null;
  }
}

/** Best-effort kick to the daemon to reload `[setup]` / `[policy]`
 *  without restart. Silently no-ops when the daemon is unreachable
 *  (the next daemon start will pick up the change anyway). */
export async function refresh(): Promise<{ refreshed: boolean; detail: string }> {
  try {
    return await invoke<{ refreshed: boolean; detail: string }>("policy_refresh");
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return { refreshed: false, detail: stringErr(err) };
  }
}

/** Fetch (or synthesise) Deep-Run defaults derived from the active
 *  bundle. Used by the Deep-Run start popover. */
export async function loadDeepRunDefaults(): Promise<DeepRunDefaults | null> {
  try {
    const defaults = await invoke<DeepRunDefaults>("deep_run_defaults");
    await patch(() => ({ deepRunDefaults: defaults, lastError: null }));
    return defaults;
  } catch (err) {
    await patch(() => ({ lastError: stringErr(err) }));
    return null;
  }
}

function stringErr(err: unknown): string {
  return typeof err === "string" ? err : String(err);
}
