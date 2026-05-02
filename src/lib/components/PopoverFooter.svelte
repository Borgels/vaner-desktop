<!--
  PopoverFooter — fixed bottom bar with the Details button + engine-health
  strip. Pause is disabled per WS0.3 (daemon endpoint not yet shipped).

  Mirrors `Popover/PopoverFooter.swift` lines 80–134. Companion window opens
  via `menu:open-companion` Tauri event (same event the tray's "Show
  Companion…" item fires) so the Rust side owns window-open ceremony.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import VStateBadge, { type VState } from "$lib/components/primitives/VStateBadge.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";

  type Props = {
    /** Engine health strip dot color. */
    health?: VState;
    healthLabel?: string;
    /** Disable Details when there's nothing the companion would surface
     *  (engineMissing, notInstalled). */
    detailsDisabled?: boolean;
    /** Override which pane the companion lands on. Defaults to Prepared. */
    detailsTab?: string | null;
  };
  const {
    health = "on",
    healthLabel,
    detailsDisabled = false,
    detailsTab = null,
  }: Props = $props();

  function openCompanion() {
    invoke("open_companion", { tab: detailsTab }).catch((e) => {
      // Fail silently — the daemon is still alive; the user can retry.
      console.warn("open_companion failed", e);
    });
  }
</script>

<div class="popover-footer">
  <V1GhostButton title="Details" disabled={detailsDisabled} onclick={openCompanion} />
  <span class="health">
    <VStateBadge state={health} size={6} />
    {#if healthLabel}<span>{healthLabel}</span>{/if}
  </span>
</div>

<style>
  .popover-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    padding: 10px 14px;
    background: rgba(0, 0, 0, 0.18);
    border-top: 0.5px solid var(--vd-hair);
  }
  .health {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-family: var(--vd-font);
    font-size: 11px;
    color: var(--vd-fg-3);
  }
</style>
