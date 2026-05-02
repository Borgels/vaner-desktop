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
    boostEngineStatusPolling,
    setSourcesCount,
    startEngineStatusPolling,
    stopEngineStatusPolling,
  } from "$lib/stores/engine-status.js";
  import {
    startAgentDetectorPolling,
    stopAgentDetectorPolling,
  } from "$lib/stores/agent-detector.js";
  import { listen, type UnlistenFn } from "@tauri-apps/api/event";
  import { showToast } from "$lib/stores/toast.js";

  type BringUpOutcome = "already_running" | "started" | "failed" | "no_workspace";
  type BringUpEvent = {
    outcome: BringUpOutcome;
    workspace: string | null;
    detail: string;
  };

  let bringUpUnlisten: UnlistenFn | null = null;

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

    // Listen for the startup auto-bring-up result. The Rust side
    // shells `vaner up --detach` itself when the cockpit is down; we
    // boost the engine_status poll to 500ms so the popover flips out
    // of .error within half a second of cockpit-up, and surface a
    // toast on failure so the user has something to act on.
    bringUpUnlisten = await listen<BringUpEvent>("engine:bring-up", (event) => {
      const result = event.payload;
      if (result.outcome === "started") {
        boostEngineStatusPolling(15_000);
      } else if (result.outcome === "failed") {
        boostEngineStatusPolling(15_000);
        showToast(
          result.detail || "Vaner could not start the engine.",
          "attention",
          5000,
        );
      }
      // already_running and no_workspace are silent — the popover
      // surfaces the right state on its own.
    });

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
    bringUpUnlisten?.();
  });
</script>

{@render children?.()}
