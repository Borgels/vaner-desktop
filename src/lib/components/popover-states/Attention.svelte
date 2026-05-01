<!--
  Attention — conflict in stored context. Red rail, evidence behind a
  disclosure. Mirrors AttentionView.swift + handoff QuietPopoverAttention.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { ConflictSummary } from "$lib/state/types.js";

  type Props = { conflict: ConflictSummary };
  const { conflict }: Props = $props();

  let showEvidence = $state(false);
</script>

<QuietShell markState="attention" stateLabel="Needs you" stateLabelTint="var(--vd-st-attention)">
  <div class="rail">
    <V1Kicker text="Conflict in stored context" color="var(--vd-st-attention)" />
    <div class="gap-6"></div>
    <V1Headline text={conflict.headline} />
  </div>

  {#if !showEvidence && conflict.evidence}
    <div class="actions">
      <V1PrimaryButton
        title="Show me the evidence"
        tint="var(--vd-st-attention)"
        onclick={() => (showEvidence = true)}
      />
    </div>
  {/if}

  {#if showEvidence && conflict.evidence}
    <div class="evidence">
      <div class="side">
        <V1Kicker text={conflict.evidence.sideALabel} />
        <pre>{conflict.evidence.sideASnippet}</pre>
      </div>
      <div class="side">
        <V1Kicker text={conflict.evidence.sideBLabel} />
        <pre>{conflict.evidence.sideBSnippet}</pre>
      </div>
    </div>
    <div class="actions">
      <V1GhostButton title="Mark A as current" />
      <V1GhostButton title="Mark B as current" />
    </div>
  {/if}

  <V1Body muted>
    You can silence this type of alert in <em>Preferences → Notifications</em>.
  </V1Body>

  {#snippet footer()}
    <PopoverFooter health="attention" healthLabel="Conflict pending" />
  {/snippet}
</QuietShell>

<style>
  .rail {
    border-left: 2px solid var(--vd-st-attention);
    padding-left: 10px;
    margin-bottom: 14px;
  }
  .gap-6 { height: 6px; }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 12px;
    flex-wrap: wrap;
  }
  .evidence {
    margin-top: 14px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .side {
    background: rgba(255, 255, 255, 0.03);
    border: 0.5px solid var(--vd-hair);
    border-radius: 7px;
    padding: 10px 12px;
  }
  pre {
    margin: 6px 0 0;
    font-family: var(--vd-font-mono);
    font-size: 11px;
    color: var(--vd-fg-2);
    white-space: pre-wrap;
    word-break: break-word;
  }
</style>
