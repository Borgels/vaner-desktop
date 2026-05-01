<!--
  Error — engine unreachable or in a degraded state. Mirrors
  ErrorView.swift + handoff V1Error.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { EngineError } from "$lib/state/types.js";
  import { invoke } from "@tauri-apps/api/core";
  import { showToast } from "$lib/stores/toast.js";

  type Props = { engine: EngineError };
  const { engine }: Props = $props();

  let restarting = $state(false);

  async function restartEngine() {
    if (restarting) return;
    restarting = true;
    try {
      const result = await invoke<string>("diagnostics_restart_engine");
      showToast(result || "Vaner restart requested.", "success", 3500);
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
</script>

<QuietShell markState="attention" stateLabel="Engine unavailable" stateLabelTint="var(--vd-st-attention)">
  <V1Kicker text="Engine error" color="var(--vd-st-attention)" />
  <div class="gap-6"></div>
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
    <PopoverFooter health="attention" healthLabel="Engine unreachable" />
  {/snippet}
</QuietShell>

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
