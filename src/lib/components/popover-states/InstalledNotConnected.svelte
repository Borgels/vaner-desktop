<!--
  InstalledNotConnected — engine is up but no sources are configured.
  First-run "connect a source" prompt. Mirrors InstalledNotConnectedView.swift
  + handoff V1Empty.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import SourceGlyph from "$lib/components/primitives/SourceGlyph.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";

  const sources = [
    { kind: "github", label: "GitHub", detail: "Repos, PRs, issues" },
    { kind: "files", label: "Local files", detail: "Code + notes" },
    { kind: "calendar", label: "Calendar", detail: "Today + next 7d" },
    { kind: "slack", label: "Slack", detail: "Threads you're in" },
  ] as const;
</script>

<QuietShell markState="idle" stateLabel="Connect a source">
  <V1Kicker text="Engine is running" />
  <div class="gap-6"></div>
  <V1Headline text="What should Vaner read?" />
  <div class="gap-8"></div>
  <V1Body muted text="Pick a source to start. You can change or add more later from Companion → Sources." />

  <div class="grid">
    {#each sources as s (s.kind)}
      <button type="button" class="cell">
        <SourceGlyph kind={s.kind} size={20} />
        <span class="cell-label">{s.label}</span>
        <span class="cell-detail">{s.detail}</span>
      </button>
    {/each}
  </div>

  <div class="actions">
    <V1PrimaryButton title="Browse all sources" />
    <V1GhostButton title="Skip for now" />
  </div>

  {#snippet footer()}
    <PopoverFooter health="idle" healthLabel="Engine running, no sources" />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .gap-8 { height: 8px; }
  .grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
    margin-top: 14px;
  }
  .cell {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
    padding: 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    cursor: pointer;
    color: var(--vd-fg-1);
    font-family: var(--vd-font);
    text-align: left;
    transition: background 0.12s;
  }
  .cell:hover { background: var(--vd-bg-2); }
  .cell-label { font-size: 13px; font-weight: 500; }
  .cell-detail { font-size: 11px; color: var(--vd-fg-3); }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
