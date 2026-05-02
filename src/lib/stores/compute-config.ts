// compute-config.ts — mirrors the Rust `engine_config_*` Tauri
// commands so the Preferences pane has a single store the
// Performance card binds against.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface ComputeConfig {
  device: string;
  cpu_fraction: number;
  gpu_memory_fraction: number;
  idle_only: boolean;
  idle_cpu_threshold: number;
  idle_gpu_threshold: number;
  max_cycle_seconds: number;
}

export type ComputePreset = "light" | "balanced" | "performance";

const internal = writable<ComputeConfig | null>(null);
export const computeConfig: Readable<ComputeConfig | null> = {
  subscribe: internal.subscribe,
};

export async function loadComputeConfig(): Promise<ComputeConfig | null> {
  try {
    const cfg = await invoke<ComputeConfig>("compute_config_get");
    internal.set(cfg);
    return cfg;
  } catch {
    internal.set(null);
    return null;
  }
}

/** Set a single key. The Rust side gates on a whitelist; only the
 *  keys actually surfaced in Preferences are accepted. */
export async function setComputeKey(key: string, value: string): Promise<ComputeConfig> {
  const cfg = await invoke<ComputeConfig>("compute_config_set", { key, value });
  internal.set(cfg);
  return cfg;
}

/** Apply one of the three named presets. The Rust side writes each
 *  key sequentially so the resulting `[compute]` block is internally
 *  consistent (parallel writes against the same config.toml race). */
export async function applyComputePreset(preset: ComputePreset): Promise<ComputeConfig> {
  const cfg = await invoke<ComputeConfig>("compute_apply_preset", { preset });
  internal.set(cfg);
  return cfg;
}

/** Classify a live config as one of the three named presets.
 *
 *  The exact-match version flagged "Custom" for any config that
 *  didn't hit each knob bang on, which made a fresh `vaner setup
 *  apply` look unconfigured (the CLI's catalogue uses subtly
 *  different defaults from the desktop's preset writer). Users want
 *  to see "Balanced is selected" the instant the wizard finishes —
 *  classify by behaviour, not by exact knob values:
 *
 *    - `idle_only` off → Performance (continuous loop)
 *    - `idle_only` on,  `max_cycle ≤ 200`  → Light (short bursts)
 *    - `idle_only` on,  otherwise          → Balanced (default)
 *
 *  Any sensible config lands on one of the three; `null` only when
 *  we have no config to read at all. */
export function classifyPreset(cfg: ComputeConfig | null): ComputePreset | null {
  if (!cfg) return null;
  if (!cfg.idle_only) return "performance";
  if (cfg.max_cycle_seconds <= 200) return "light";
  return "balanced";
}
