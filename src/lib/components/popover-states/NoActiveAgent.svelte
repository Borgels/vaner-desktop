<!--
  NoActiveAgent — moments are ready but no agent is running to receive them.
  Mirrors NoActiveAgentView.swift + handoff V1NoAgent.
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
  import type { AgentSuggestion } from "$lib/state/types.js";

  type Props = { pendingCount: number; suggestedLaunch: AgentSuggestion[] };
  const { pendingCount, suggestedLaunch }: Props = $props();
</script>

<QuietShell markState="prepared" stateLabel="No agent open" stateLabelTint="var(--vd-amber)">
  <V1Kicker text="Prepared · waiting for you" color="var(--vd-amber)" />
  <div class="gap-6"></div>
  <V1Headline text={`I have ${pendingCount} ${pendingCount === 1 ? "moment" : "moments"} ready, but no AI agent is running.`} />
  <div class="gap-8"></div>
  <V1Body muted text="Launch one to send them over, or copy / save manually." />

  {#if suggestedLaunch.length > 0}
    <div class="grid">
      {#each suggestedLaunch as a (a.id)}
        <button class="agent" type="button">
          <SourceGlyph kind="agent" size={18} />
          <span>{a.displayName}</span>
        </button>
      {/each}
    </div>
  {/if}

  <div class="actions">
    <V1GhostButton title="Copy to clipboard" />
    <V1GhostButton title="Save as file" />
  </div>

  {#snippet footer()}
    <PopoverFooter health="prepared" healthLabel="Holding for an agent" />
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
  .agent {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    cursor: pointer;
    color: var(--vd-fg-1);
    font-family: var(--vd-font);
    font-size: 13px;
    text-align: left;
    transition: background 0.12s;
  }
  .agent:hover { background: var(--vd-bg-2); }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
