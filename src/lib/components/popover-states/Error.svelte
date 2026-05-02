<!--
  Error — engine unreachable or in a degraded state. Mirrors
  ErrorView.swift + handoff V1Error.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { EngineError } from "$lib/state/types.js";
  import { invoke } from "@tauri-apps/api/core";
  import { showToast } from "$lib/stores/toast.js";
  import { boostEngineStatusPolling } from "$lib/stores/engine-status.js";
  import { loadWorkspace, workspacePath } from "$lib/stores/workspace.js";

  type Props = { engine: EngineError };
  const { engine }: Props = $props();

  // The reducer hits this branch any time the cockpit is silent. Two
  // very different stories live here:
  //
  //   - The user has a workspace selected and the engine should be
  //     running for it — that *is* an error, scary copy is right.
  //   - The user hasn't picked a workspace yet (the desktop is fresh,
  //     or they cleared it) — engine being down is the expected idle
  //     state, not a failure. Calling that "ENGINE UNAVAILABLE"
  //     misleads them into thinking something broke. We soften the
  //     copy and steer them at setup instead of "Restart engine".
  onMount(() => void loadWorkspace());
  const unconfigured = $derived($workspacePath == null);

  type BringUpOutcome = "already_running" | "started" | "failed" | "no_workspace";
  type BringUpResult = {
    outcome: BringUpOutcome;
    workspace: string | null;
    detail: string;
  };

  let restarting = $state(false);

  async function restartEngine() {
    if (restarting) return;
    restarting = true;
    // Boost the engine_status poll up front so the moment the cockpit
    // answers we flip out of .error within 500ms, regardless of the
    // bring-up RPC's own latency.
    boostEngineStatusPolling(15_000);
    try {
      const result = await invoke<BringUpResult>("bring_up_engine");
      if (result.outcome === "started" || result.outcome === "already_running") {
        showToast(
          result.outcome === "started" ? "Vaner engine started." : "Vaner engine already running.",
          "success",
          3000,
        );
      } else {
        showToast(
          result.detail || "Vaner could not start the engine.",
          "attention",
          5000,
        );
      }
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Failed to restart Vaner: ${err}`,
        "attention",
        4000,
      );
    } finally {
      restarting = false;
    }
  }

  async function openDiagnostics() {
    try {
      // Reuse the companion window's deep-link path. The Rust side
      // listens for the `companion:navigate` event and switches the
      // pane; opening directly with ?tab=diagnostics works equally well
      // when the window is created fresh.
      await invoke("open_companion", { tab: "diagnostics" });
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Failed to open diagnostics: ${err}`,
        "attention",
        4000,
      );
    }
  }

  async function openSetup() {
    try {
      await invoke("open_onboarding");
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not open setup: ${err}`,
        "attention",
        4000,
      );
    }
  }
</script>

{#if unconfigured}
  <QuietShell markState="idle" stateLabel="Setup not finished">
    <V1Headline text="Vaner isn't running yet." />
    <V1Body
      muted
      text="No workspace is set, so the engine isn't watching anything. Finish setup or open a wired agent in a repo and Vaner will start."
    />
    <div class="actions">
      <V1PrimaryButton title="Run setup" onclick={openSetup} />
      <V1GhostButton title="Diagnostics" onclick={openDiagnostics} />
    </div>
    {#snippet footer()}
      <PopoverFooter health="idle" healthLabel="Idle — no workspace yet" />
    {/snippet}
  </QuietShell>
{:else}
  <QuietShell markState="attention" stateLabel="Engine unavailable" stateLabelTint="var(--vd-st-attention)">
    <V1Headline text={engine.message} />

    <div class="info">
      <V1Body muted>What's still working:</V1Body>
      <ul>
        <li>Recently prepared moments stay in the popover</li>
        <li>Sending context to your agent still works</li>
        <li>Preferences are intact</li>
      </ul>
    </div>

    {#if engine.incidentID || engine.port}
      <pre class="meta">{engine.incidentID ? `incident ${engine.incidentID}` : ""}{engine.port ? `  port ${engine.port}` : ""}</pre>
    {/if}

    <div class="actions">
      <V1PrimaryButton
        title={restarting ? "Restarting…" : "Restart engine"}
        tint="var(--vd-st-attention)"
        onclick={restartEngine}
      />
      <V1GhostButton title="Diagnostics" onclick={openDiagnostics} />
    </div>

    {#snippet footer()}
      <PopoverFooter health="attention" />
    {/snippet}
  </QuietShell>
{/if}

<style>
  .gap-6 { height: 6px; }
  .info { margin-top: 14px; }
  .info ul {
    margin: 8px 0 0;
    padding-left: 18px;
    list-style: disc;
    color: var(--vd-fg-2);
    font-family: var(--vd-font);
    font-size: 12px;
    line-height: 1.5;
  }
  .meta {
    margin: 12px 0 0;
    padding: 6px 8px;
    background: rgba(0, 0, 0, 0.25);
    border-radius: 6px;
    font-family: var(--vd-font-mono);
    font-size: 10.5px;
    color: var(--vd-fg-3);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
