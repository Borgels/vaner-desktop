<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import { bootstrapUpdaterListeners } from "$lib/stores/updater.js";
  import { loadStatus } from "$lib/stores/setup.js";

  let { children } = $props();

  onMount(async () => {
    // Only the main popover window owns the cross-window orchestration
    // (event listeners, first-launch onboarding kickoff). The companion
    // and onboarding windows mount this same layout but skip everything
    // here.
    const label = getCurrentWebviewWindow().label;
    if (label !== "main") return;

    bootstrapAppStateListeners();
    bootstrapUpdaterListeners();

    // First-run check: if no setup has completed, open the dedicated
    // onboarding window. The popover keeps rendering its current state
    // (engineMissing / installedNotConnected / etc.) while the user
    // runs through onboarding in the second window. On completion the
    // onboarding side calls close_onboarding and the layout never
    // re-fires this branch.
    try {
      const status = await loadStatus();
      const completedAt = status?.setup?.completed_at;
      if (!completedAt) {
        await invoke("open_onboarding").catch(() => {});
      }
    } catch {
      // Daemon / CLI unreachable: leave the popover on its default
      // state. The user can re-run setup later from the companion
      // window's Engine pane.
    }
  });
</script>

{@render children?.()}
