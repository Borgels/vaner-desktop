// 0.8.6 WS-DESK-LINUX — Hand-mirrored types for the Simple-Mode setup
// surface (engine modules under `src/vaner/setup/` in Vaner).
//
// TODO: replace with ts-rs generated types when vaner-contract setup
// types ship (the existing 0.8.5 ts-rs codegen covers prediction /
// resolution shapes; setup types are a separate follow-up PR per
// 0.8.6 WS11). Source-of-truth references in the Vaner repo:
//
// - src/vaner/setup/enums.py
// - src/vaner/setup/answers.py
// - src/vaner/setup/policy.py
// - src/vaner/setup/select.py        (SelectionResult)
// - src/vaner/setup/apply.py         (AppliedPolicy)
// - src/vaner/setup/hardware.py      (HardwareProfile)
// - src/vaner/cli/commands/setup.py  (_bundle_to_dict / _selection_to_dict / _hardware_to_dict)
//
// When the ts-rs PR lands, rsync overwrites this file with the
// generated copy. Until then, keep the Literal sets in lock-step.

// ---------------------------------------------------------------------
// Outcome-level question vocabulary (Simple-Mode wizard).
// Mirrors src/vaner/setup/enums.py.
// ---------------------------------------------------------------------

export type WorkStyle =
  | "writing"
  | "research"
  | "planning"
  | "support"
  | "learning"
  | "coding"
  | "general"
  | "mixed"
  | "unsure";

export type Priority =
  | "balanced"
  | "speed"
  | "quality"
  | "privacy"
  | "cost"
  | "low_resource";

export type ComputePosture = "light" | "balanced" | "available_power";

export type CloudPosture =
  | "local_only"
  | "ask_first"
  | "hybrid_when_worth_it"
  | "best_available";

export type BackgroundPosture =
  | "minimal"
  | "normal"
  | "idle_more"
  | "deep_run_aggressive";

export type HardwareTier =
  | "light"
  | "capable"
  | "high_performance"
  | "unknown";

// ---------------------------------------------------------------------
// SetupAnswers — input to the recommend / apply surfaces.
// Mirrors src/vaner/setup/answers.py (work_styles is a list on the wire).
// ---------------------------------------------------------------------

export interface SetupAnswers {
  work_styles: WorkStyle[];
  priority: Priority;
  compute_posture: ComputePosture;
  cloud_posture: CloudPosture;
  background_posture: BackgroundPosture;
}

// ---------------------------------------------------------------------
// VanerPolicyBundle — the bundle catalogue entry; one of the seven in
// PROFILE_CATALOG. Mirrors src/vaner/setup/policy.py.
// ---------------------------------------------------------------------

export type LocalCloudPosture =
  | "local_only"
  | "local_preferred"
  | "hybrid"
  | "cloud_preferred";

export type RuntimeProfile = "small" | "medium" | "large" | "auto";
export type SpendProfile = "zero" | "low" | "medium" | "high";
export type LatencyProfile = "snappy" | "balanced" | "quality";
export type PrivacyProfile = "strict" | "standard" | "relaxed";

export type ContextInjectionMode =
  | "none"
  | "digest_only"
  | "adopted_package_only"
  | "top_match_auto_include"
  | "policy_hybrid"
  | "client_controlled";

export type DeepRunPreset =
  | "balanced"
  | "research"
  | "writing"
  | "support"
  | "planning"
  | "learning"
  | "coding";

export type PredictionHorizonKey =
  | "likely_next"
  | "long_horizon"
  | "finish_partials"
  | "balanced";

export type PredictionHorizonBias = Record<PredictionHorizonKey, number>;

export interface VanerPolicyBundle {
  id: string;
  label: string;
  description: string;
  local_cloud_posture: LocalCloudPosture;
  runtime_profile: RuntimeProfile;
  spend_profile: SpendProfile;
  latency_profile: LatencyProfile;
  privacy_profile: PrivacyProfile;
  prediction_horizon_bias: PredictionHorizonBias;
  drafting_aggressiveness: number;
  exploration_ratio: number;
  persistence_strength: number;
  goal_weighting: number;
  context_injection_default: ContextInjectionMode;
  deep_run_profile: DeepRunPreset;
}

// ---------------------------------------------------------------------
// SelectionResult — output of `vaner setup recommend --json`.
// Mirrors src/vaner/setup/select.py + _selection_to_dict.
// ---------------------------------------------------------------------

export interface SelectionResult {
  bundle: VanerPolicyBundle;
  score: number;
  reasons: string[];
  runner_ups: VanerPolicyBundle[];
  forced_fallback: boolean;
}

// ---------------------------------------------------------------------
// AppliedPolicy — output of `vaner setup apply --json` (status object).
// The CLI returns { config_path, selected_bundle_id, reasons, daemon }.
// We wrap it here for the desktop app and also surface the
// widens-cloud-posture flag the app needs for the confirm dialog.
// ---------------------------------------------------------------------

export interface AppliedPolicy {
  config_path: string;
  selected_bundle_id: string;
  reasons: string[];
  /** True when the new bundle's local_cloud_posture is strictly more
   *  permissive than the previous one's. The desktop app must confirm
   *  before re-calling apply with confirm_cloud_widening=true. The
   *  engine emits a `WIDENS_CLOUD_POSTURE` sentinel in
   *  AppliedPolicy.overrides_applied; the Tauri shell parses that and
   *  surfaces this boolean. */
  widens_cloud_posture: boolean;
  /** When `widens_cloud_posture` is true and `confirm_cloud_widening`
   *  was not passed, the engine declines to write. The desktop app
   *  must re-call apply with the confirm flag set. */
  written: boolean;
  daemon: { reachable: boolean; detail?: string };
}

// ---------------------------------------------------------------------
// HardwareProfile — output of `vaner setup hardware --json`.
// Mirrors _hardware_to_dict in src/vaner/cli/commands/setup.py.
// ---------------------------------------------------------------------

export type HostOS = "linux" | "darwin" | "windows";
export type CPUClass = "low" | "mid" | "high";
export type GPU = "none" | "integrated" | "nvidia" | "amd" | "apple_silicon";
export type Runtime = "ollama" | "llama.cpp" | "lmstudio" | "vllm" | "mlx";

export interface GPUDevice {
  name: string;
  vendor: string;
  kind: GPU;
  memory_total_bytes: number | null;
  memory_display_gb: number | null;
  memory_kind: "vram" | "unified" | "system" | "unknown";
}

export interface HardwareProfile {
  os: HostOS;
  cpu_class: CPUClass;
  ram_gb: number;
  memory_total_bytes?: number;
  memory_display_gb?: number;
  memory_is_unified?: boolean;
  gpu: GPU;
  gpu_vram_gb: number | null;
  gpu_devices?: GPUDevice[];
  is_battery: boolean;
  thermal_constrained: boolean;
  detected_runtimes: Runtime[];
  /** Each row is a `[runtime, model_id, size_label]` tuple. */
  detected_models: [string, string, string][];
  tier: HardwareTier;
}

// ---------------------------------------------------------------------
// SetupQuestion — schema for the question pickers. Hand-mirrored from
// the choice tuples in src/vaner/cli/commands/setup.py until the
// daemon ships `GET /setup/questions`.
// ---------------------------------------------------------------------

export interface SetupChoice {
  value: string;
  label: string;
  hint?: string;
}

export interface SetupQuestion {
  id:
    | "work_styles"
    | "priority"
    | "compute_posture"
    | "cloud_posture"
    | "background_posture";
  label: string;
  prompt: string;
  multi: boolean;
  choices: SetupChoice[];
}

// ---------------------------------------------------------------------
// SetupStatus — output of `vaner setup show --json`.
// CLI emits { repo_root, setup, policy, hardware, applied_policy, bundle }.
// We expose a slightly normalised shape so the Svelte layer can read
// `completed_at` directly without poking into the raw `[setup]` table.
// ---------------------------------------------------------------------

export interface SetupStatus {
  repo_root: string;
  setup: {
    mode?: "simple" | "advanced";
    work_styles?: WorkStyle[];
    priority?: Priority;
    compute_posture?: ComputePosture;
    cloud_posture?: CloudPosture;
    background_posture?: BackgroundPosture;
    completed_at?: string | null;
    version?: number;
  };
  policy: {
    selected_bundle_id?: string;
    bundle_overrides?: Record<string, unknown>;
    auto_select?: boolean;
  };
  hardware: HardwareProfile;
  applied_policy: {
    bundle_id?: string;
    overrides_applied?: string[];
    error?: string;
  };
  bundle: VanerPolicyBundle | null;
}

// ---------------------------------------------------------------------
// DeepRunDefaults — output of (future) `GET /deep-run/defaults`. Until
// the endpoint ships the desktop derives a synthetic record from the
// active bundle, so this shape is the union of "everything we might
// surface in the start-deep-run dialog".
// ---------------------------------------------------------------------

export interface DeepRunDefaults {
  preset: DeepRunPreset;
  locality: "local_only" | "local_preferred" | "hybrid" | "cloud_preferred";
  cost_cap_usd: number | null;
  horizon_bias: PredictionHorizonKey;
  /** "Deep-Run prepares; it does not act." (spec §13.5). The desktop
   *  surfaces this on the start dialog; if the engine ships a localised
   *  string we adopt it instead. */
  reminder: string;
}
