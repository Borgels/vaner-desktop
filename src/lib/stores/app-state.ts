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

  await listen<void>("menu:open-preferences", () => {
    // Preferences window wiring comes in a follow-up; for now we
    // acknowledge the click so the user knows the menu item is
    // reachable.
    showToast("Preferences pane is coming soon.", "info", 4000);
  });

  await listen<void>("setup:appindicator-missing", () => {
    needsAppIndicator.set(true);
  });
}
