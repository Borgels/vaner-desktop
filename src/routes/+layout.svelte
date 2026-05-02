<script lang="ts">
  import "../app.css";
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import { bootstrapUpdaterListeners } from "$lib/stores/updater.js";
  import { loadStatus } from "$lib/stores/setup.js";
  import { loadWorkspace, workspacePath } from "$lib/stores/workspace.js";
  import { get } from "svelte/store";
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

    // Hydrate the workspace store before anything that shells the CLI.
    // The reducer reads this synchronously to decide between
    // .needsWorkspace and the rest of the chain; the engine-status
    // poll downstream feeds `--path` from the same source.
    await loadWorkspace();

    // Reducer-input polling. Both are idempotent; the popover survives
    // these returning errors (the stores keep their last value).
    startEngineStatusPolling();
    startAgentDetectorPolling();

    // First-run check: if no setup has completed, open the dedicated
    // onboarding window. Gated on a workspace being picked — without
    // one, `vaner setup show --path .` runs against the desktop's cwd
    // (often /) and reports an empty [setup], firing onboarding on
    // every launch. The .needsWorkspace popover state takes the user
    // through the picker first.
    if (!get(workspacePath)) return;
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
