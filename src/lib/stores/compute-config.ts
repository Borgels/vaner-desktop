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

/** Heuristic: classify a live config as one of the named presets.
 *  Used to highlight the active preset in the Preferences card.
 *  Falls back to `null` when the config doesn't match any preset
 *  exactly — happens when the user has nudged a slider after a
 *  preset apply, which is fine ("custom" lights nothing). */
export function classifyPreset(cfg: ComputeConfig | null): ComputePreset | null {
  if (!cfg) return null;
  const eq = (a: number, b: number) => Math.abs(a - b) < 1e-3;
  if (eq(cfg.cpu_fraction, 0.15) && cfg.idle_only && eq(cfg.max_cycle_seconds, 180)) return "light";
  if (eq(cfg.cpu_fraction, 0.25) && cfg.idle_only && eq(cfg.max_cycle_seconds, 300)) return "balanced";
  if (eq(cfg.cpu_fraction, 0.5) && !cfg.idle_only && eq(cfg.max_cycle_seconds, 600)) return "performance";
  return null;
}
