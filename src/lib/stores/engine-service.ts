// engine-service.ts — mirrors the Rust-side `engine_service_*` Tauri
// commands so the Preferences pane has a single store to bind a
// toggle against. The systemd-user `vaner-engine.service` install
// flow is opt-in and per-user; auto-bring-up at app launch handles
// the more common case where users only want the engine running
// while the desktop is open.

import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";

export type ServiceState = "unavailable" | "missing" | "disabled" | "enabled" | "active";

export interface ServiceStatus {
  state: ServiceState;
  workspace?: string;
  unit_path: string;
  /** Whether the user manager keeps running across logout (i.e.
   *  `loginctl enable-linger` was applied). Without this, the service
   *  stops as soon as the user logs out. */
  linger_enabled: boolean;
  detail?: string;
}

const internal = writable<ServiceStatus | null>(null);
export const engineService: Readable<ServiceStatus | null> = {
  subscribe: internal.subscribe,
};

export async function loadEngineServiceStatus(): Promise<ServiceStatus | null> {
  try {
    const status = await invoke<ServiceStatus>("engine_service_status");
    internal.set(status);
    return status;
  } catch {
    internal.set(null);
    return null;
  }
}

/** Install + enable + start the systemd unit. Throws on failure with
 *  a human-readable message the caller surfaces in a toast. */
export async function installEngineService(): Promise<ServiceStatus> {
  const status = await invoke<ServiceStatus>("engine_service_install");
  internal.set(status);
  return status;
}

/** Disable + stop + remove the unit file. Idempotent — calling on a
 *  Missing service is a no-op. */
export async function uninstallEngineService(): Promise<ServiceStatus> {
  const status = await invoke<ServiceStatus>("engine_service_uninstall");
  internal.set(status);
  return status;
}

/** Toggle `loginctl enable-linger / disable-linger` for the current
 *  user via `pkexec` (graphical polkit prompt). Without linger, the
 *  service stops on logout; with linger, it survives across reboots
 *  even when the user isn't logged in graphically. */
export async function setEngineServiceLinger(enable: boolean): Promise<ServiceStatus> {
  const status = await invoke<ServiceStatus>("engine_service_set_linger", { enable });
  internal.set(status);
  return status;
}
