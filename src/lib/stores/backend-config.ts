// backend-config.ts — mirrors `backend_config_get / backend_apply_preset
// / backend_classify` Tauri commands. The Models pane binds against
// these for picking which provider Vaner's ponder/answer loop calls.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export interface BackendConfig {
  name: string;
  base_url: string;
  model: string;
  api_key_env: string;
}

export type BackendPreset = "ollama" | "openai" | "anthropic" | "custom";

const internal = writable<BackendConfig | null>(null);
export const backendConfig: Readable<BackendConfig | null> = {
  subscribe: internal.subscribe,
};

export async function loadBackendConfig(): Promise<BackendConfig | null> {
  try {
    const cfg = await invoke<BackendConfig>("backend_config_get");
    internal.set(cfg);
    return cfg;
  } catch {
    internal.set(null);
    return null;
  }
}

/** Apply one of the four named presets (Ollama / OpenAI / Anthropic /
 *  Custom). Rust writes `backend.name`, `backend.base_url`,
 *  `backend.model`, `backend.api_key_env` sequentially so a partial
 *  failure leaves the previous backend pointing at consistent values
 *  rather than a half-converted Frankenstein.
 *
 *  `Custom` is a no-op on the Rust side (no template); use the
 *  per-key `setBackendKey` helper to edit fields directly. */
export async function applyBackendPreset(preset: BackendPreset): Promise<BackendConfig> {
  const cfg = await invoke<BackendConfig>("backend_apply_preset", { preset });
  internal.set(cfg);
  return cfg;
}

/** Set a single backend.* key. Reuses the compute_config_set whitelist;
 *  the Rust side accepts `backend.{name,base_url,model,api_key_env}`
 *  and rejects anything else. */
export async function setBackendKey(key: string, value: string): Promise<void> {
  await invoke("compute_config_set", { key: `backend.${key}`, value });
  await loadBackendConfig();
}

/** Classify a backend config as one of the named presets. Pure mirror
 *  of the Rust `backend_classify` so the UI can highlight the active
 *  card without round-tripping through Tauri on every render.
 *
 *  Returns `null` while the config is still loading so the picker
 *  doesn't flash the wrong card during hydration. Treats an empty
 *  `base_url` (the Python CLI's default scaffolding) as `ollama` —
 *  the desktop writes Ollama defaults on first mount, so showing
 *  any other card highlighted in that brief window would lie. */
export function classifyBackend(cfg: BackendConfig | null): BackendPreset | null {
  if (!cfg) return null;
  if (cfg.base_url.trim() === "") return "ollama";
  if (cfg.base_url.includes("api.openai.com")) return "openai";
  if (cfg.base_url.includes("api.anthropic.com")) return "anthropic";
  if (cfg.base_url.includes("11434") || cfg.base_url.includes("ollama")) return "ollama";
  return "custom";
}
