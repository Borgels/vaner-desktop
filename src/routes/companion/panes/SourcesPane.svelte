<!--
  Sources pane — list of configured sources. Add / Reconnect / Remove.
  Implementation depth in v0.2.2 is intentionally light: the daemon-side
  source-management endpoints aren't all there yet. We surface what
  setup_status returns and link to the wizard for the rest.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import SourceGlyph from "$lib/components/primitives/SourceGlyph.svelte";
  import VStateBadge from "$lib/components/primitives/VStateBadge.svelte";
  import { setup, loadStatus } from "$lib/stores/setup.js";

  onMount(() => {
    loadStatus();
  });

  // The daemon's setup_status doesn't yet enumerate connected sources
  // structurally — that's a Tier B endpoint. For v0.2.2 we surface
  // whether *any* are connected (derivable from work_styles being set)
  // and link the rest into the wizard.
  const hasSetup = $derived(Boolean($setup.status?.setup?.completed_at));
  const sources: Array<{
    id: string;
    label: string;
    kind: string;
    detail: string;
  }> = $derived(
    hasSetup
      ? [
          {
            id: "files",
            label: "Local files",
            kind: "files",
            detail: "Configured during setup wizard",
          },
        ]
      : [],
  );
</script>

<header class="hd">
  <V1Kicker text="Sources" />
  <V1Headline text="What Vaner reads for you" size={22} />
  <V1Body
    muted
    text="The richer the sources, the better Vaner can prepare. Connect more, pause individual ones, or reconnect after a token expires."
  />
</header>

{#if sources.length === 0}
  <section class="empty">
    <V1Body
      muted
      text="No sources connected yet. The setup wizard walks through the first one."
    />
    <div class="actions">
      <V1PrimaryButton title="Open setup wizard" onclick={() => (window.location.href = "/setup")} />
    </div>
  </section>
{:else}
  <section>
    <VSectionLabel text="Connected" />
    <div class="rows">
      {#each sources as s (s.id ?? s.label)}
        <div class="row">
          <SourceGlyph kind={s.kind ?? "unknown"} size={18} />
          <div class="row-body">
            <div class="row-label">{s.label ?? s.kind ?? "Unknown source"}</div>
            <div class="row-detail">{s.detail ?? "Connected"}</div>
          </div>
          <span class="row-state">
            <VStateBadge state="on" size={6} />
            <span>active</span>
          </span>
          <div class="row-actions">
            <V1GhostButton title="Sync now" />
            <V1GhostButton title="Reconnect" />
            <V1GhostButton title="Remove" destructive />
          </div>
        </div>
      {/each}
    </div>
  </section>

  <section class="add">
    <V1GhostButton title="Add another source…" onclick={() => (window.location.href = "/setup")} />
  </section>
{/if}

<style>
  .hd { display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px; }
  .rows { display: flex; flex-direction: column; gap: 8px; margin-top: 10px; }
  .row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 12px 14px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
  }
  .row-body { flex: 1 1 auto; min-width: 0; }
  .row-label { font-size: 13px; font-weight: 500; color: var(--vd-fg-1); }
  .row-detail { font-size: 11px; color: var(--vd-fg-3); margin-top: 2px; }
  .row-state {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    font-size: 11px;
    color: var(--vd-fg-2);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .row-actions { display: flex; gap: 6px; }
  .add { margin-top: 16px; }
  .empty { margin-top: 12px; }
  .actions { display: flex; gap: 6px; margin-top: 14px; }
</style>
