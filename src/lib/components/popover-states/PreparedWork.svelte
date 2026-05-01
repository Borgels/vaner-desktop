<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import { showToast } from "$lib/stores/toast.js";
  import type { PreparedWorkAction, PreparedWorkCard } from "$lib/contract/types.js";

  type Props = { cards: PreparedWorkCard[] };
  const { cards }: Props = $props();

  async function run(card: PreparedWorkCard, action: PreparedWorkAction) {
    if (!action.endpoint) return;
    try {
      await invoke("prepared_work_action", { endpoint: action.endpoint, kind: action.kind, arguments: action.arguments ?? {} });
      showToast(`${action.label} complete.`, "success", 3000);
    } catch (err) {
      const msg = typeof err === "string" ? err : `Couldn't ${action.label.toLowerCase()}.`;
      showToast(msg, "attention", 5000);
    }
  }
</script>

<QuietShell markState="active" stateLabel={`Prepared work · ${cards.length}`} stateLabelTint="var(--vd-st-active)">
  <V1Kicker text="Vaner prepared this" color="var(--vd-st-active)" />

  <div class="rows">
    {#each cards as card, i (card.id)}
      <article class="row">
        <div class="top">
          <span class="badge">{card.badge}</span>
          <div class="title">{card.title}</div>
        </div>
        <div class="summary">{card.summary}</div>
        {#if card.why_prepared}
          <div class="why">{card.why_prepared}</div>
        {/if}
        <div class="meta">
          <span>{card.confidence_label}</span>
          <span>{card.freshness_label}</span>
          <span>{card.target_label}</span>
          {#if card.evidence_count > 0}
            <span>{card.evidence_count} sources</span>
          {/if}
        </div>
        {#if card.action_note}
          <div class:warn={card.freshness_state === "possibly_stale" || card.freshness_state === "stale"} class="note">
            {card.action_note}
          </div>
        {/if}
        <div class="actions">
          {#if card.primary_action?.endpoint}
            {#if i === 0}
              <V1PrimaryButton title={card.primary_action.label} tint="var(--vd-st-active)" onclick={() => run(card, card.primary_action!)} />
            {:else}
              <V1GhostButton title={card.primary_action.label} onclick={() => run(card, card.primary_action!)} />
            {/if}
          {/if}
          {#each card.secondary_actions.filter((a) => a.endpoint).slice(0, 3) as action (`${card.id}-${action.kind}-${action.label}`)}
            <V1GhostButton title={action.label} onclick={() => run(card, action)} />
          {/each}
        </div>
      </article>
    {/each}
  </div>

  {#snippet footer()}
    <PopoverFooter health="active" healthLabel="Inspect before using" />
  {/snippet}
</QuietShell>

<style>
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }
  .row {
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
  }
  .top {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .badge {
    flex: 0 0 auto;
    padding: 2px 7px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--vd-st-active) 18%, transparent);
    color: var(--vd-st-active);
    font-family: var(--vd-font);
    font-size: 10px;
    font-weight: 600;
  }
  .title {
    min-width: 0;
    font-family: var(--vd-font);
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    line-height: 1.3;
  }
  .summary {
    margin-top: 6px;
    font-family: var(--vd-font);
    font-size: 11.5px;
    line-height: 1.38;
    color: var(--vd-fg-3);
  }
  .why,
  .note {
    margin-top: 6px;
    font-family: var(--vd-font);
    font-size: 10.8px;
    line-height: 1.35;
    color: var(--vd-fg-4);
  }
  .note.warn {
    color: var(--vd-st-warn);
  }
  .meta {
    display: flex;
    flex-wrap: wrap;
    gap: 7px;
    margin-top: 8px;
    color: var(--vd-fg-4);
    font-family: var(--vd-font-mono);
    font-size: 10.5px;
    font-variant-numeric: tabular-nums;
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 10px;
  }
</style>
