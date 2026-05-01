<script lang="ts">
  import "../app.css";
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import { bootstrapUpdaterListeners } from "$lib/stores/updater.js";
  import { loadStatus } from "$lib/stores/setup.js";
  import {
    setSourcesCount,
    startEngineStatusPolling,
    stopEngineStatusPolling,
  } from "$lib/stores/engine-status.js";
  import {
    startAgentDetectorPolling,
    stopAgentDetectorPolling,
  } from "$lib/stores/agent-detector.js";

  let { children } = $props();

  onMount(async () => {
    // Only the main popover window owns the cross-window orchestration
    // (event listeners, first-launch onboarding kickoff, polling). The
    // companion + onboarding windows mount this same layout but skip
    // everything here.
    const label = getCurrentWebviewWindow().label;
    if (label !== "main") return;

    bootstrapAppStateListeners();
    bootstrapUpdaterListeners();

    // Reducer-input polling. Both are idempotent; the popover survives
    // these returning errors (the stores keep their last value).
    startEngineStatusPolling();
    startAgentDetectorPolling();

    // First-run check: if no setup has completed, open the dedicated
    // onboarding window. The popover keeps rendering its current state
    // (engineMissing / installedNotConnected / etc.) while the user
    // runs through onboarding in the second window. On completion the
    // onboarding side calls close_onboarding and the layout never
    // re-fires this branch.
    try {
      const status = await loadStatus();
      const completedAt = status?.setup?.completed_at;
      // Also overlay sourcesCount onto the engine-status store so the
      // reducer can tell .installedNotConnected from .watching. The
      // SetupStatus shape doesn't yet structurally enumerate sources,
      // so we use completed_at as a proxy: completed setup → ≥1 source.
      setSourcesCount(completedAt ? 1 : 0);

      if (!completedAt) {
        await invoke("open_onboarding").catch(() => {});
      }
    } catch {
      // Daemon / CLI unreachable: leave the popover on its default
      // state. The user can re-run setup later from the companion
      // window's Engine pane.
    }
  });

  onDestroy(() => {
    stopEngineStatusPolling();
    stopAgentDetectorPolling();
  });
</script>

{@render children?.()}
