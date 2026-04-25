import { writable } from "svelte/store";
import { listen } from "@tauri-apps/api/event";
import { showToast } from "./toast.js";

/**
 * Global app-level flags driven by Tauri events fired from the tray
 * menu (`menu:toggle-pause`, `menu:open-preferences`) and startup
 * session detection (`setup:appindicator-missing`).
 *
 * The Svelte UI subscribes to these stores and reacts — no direct
 * access to Tauri events from components.
 */
export const isPaused = writable<boolean>(false);
export const needsAppIndicator = writable<boolean>(false);

let bootstrapped = false;

export async function bootstrapAppStateListeners(): Promise<void> {
  if (bootstrapped) return;
  bootstrapped = true;

  await listen<void>("menu:toggle-pause", () => {
    isPaused.update((p) => {
      const next = !p;
      showToast(next ? "Vaner paused" : "Vaner resumed", "info");
      return next;
    });
  });

  await listen<void>("menu:open-preferences", async () => {
    // 0.8.5 WS12: navigate to /preferences (lands on the MCP Clients
    // tab). Loaded lazily so this store has no SvelteKit `goto`
    // dependency at module-load time.
    const { goto } = await import("$app/navigation");
    void goto("/preferences");
  });

  await listen<void>("setup:appindicator-missing", () => {
    needsAppIndicator.set(true);
  });
}
