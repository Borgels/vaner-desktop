<!--
  PermissionNeeded — one or more sources blocked. Mirrors
  PermissionNeededView.swift + handoff V1Permission.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import SourceGlyph from "$lib/components/primitives/SourceGlyph.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { SourceStatus } from "$lib/state/types.js";

  type Props = { sources: SourceStatus[] };
  const { sources }: Props = $props();
  const headline = $derived(
    sources.length === 1
      ? `${sources[0].source.label} needs to reconnect.`
      : `${sources.length} sources need to reconnect.`,
  );
</script>

<QuietShell markState="attention" stateLabel="Needs access" stateLabelTint="var(--vd-st-attention)">
  <V1Kicker text="Permission required" color="var(--vd-st-attention)" />
  <div class="gap-6"></div>
  <V1Headline text={headline} />

  <div class="rows">
    {#each sources as s (s.source.id)}
      <div class="row">
        <SourceGlyph kind={s.source.kind} size={14} />
        <div class="row-body">
          <div class="row-label">{s.source.label}</div>
          <div class="row-detail">{s.detail}</div>
        </div>
        <span class="row-tag">{s.status}</span>
      </div>
    {/each}
  </div>

  <div class="actions">
    <V1PrimaryButton title="Reconnect" />
    <V1GhostButton title="Remove source" destructive />
  </div>

  {#snippet footer()}
    <PopoverFooter health="attention" healthLabel="Other sources are still working" />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 14px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: color-mix(in srgb, var(--vd-st-attention) 6%, transparent);
    border: 0.5px solid color-mix(in srgb, var(--vd-st-attention) 30%, transparent);
    border-radius: 7px;
  }
  .row-body { flex: 1 1 auto; min-width: 0; }
  .row-label {
    font-family: var(--vd-font);
    font-size: 12px;
    font-weight: 500;
    color: var(--vd-fg-1);
  }
  .row-detail {
    font-family: var(--vd-font);
    font-size: 11px;
    color: var(--vd-fg-3);
  }
  .row-tag {
    flex: 0 0 auto;
    font-family: var(--vd-font);
    font-size: 10px;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--vd-st-attention);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
