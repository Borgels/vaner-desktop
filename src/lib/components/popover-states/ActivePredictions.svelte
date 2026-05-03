<!--
  ActivePredictions — 0.8.0 prediction-centric pondering. Each row is a
  PredictedPrompt; the lead row gets the Adopt primary button. Mirrors
  ActivePredictionsView.swift.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import PopoverContextBlock from "./PopoverContextBlock.svelte";
  import PopoverQuickActions from "./PopoverQuickActions.svelte";
  import { showToast } from "$lib/stores/toast.js";
  import type { PredictedPrompt } from "$lib/contract/types.js";
  import { isAdoptable } from "$lib/contract/types.js";
  import type { PopoverRuntimeContext } from "$lib/state/types.js";

  type Props = { predictions: PredictedPrompt[]; context: PopoverRuntimeContext };
  const { predictions, context }: Props = $props();
  const visiblePredictions = $derived(predictions.slice(0, 4));

  function typeLabel(p: PredictedPrompt): string {
    switch (p.spec.hypothesis_type) {
      case "likely_next": return "next step";
      case "possible_branch": return "opportunity";
      case "long_tail": return "watch";
      default: return p.source_label ?? "prediction";
    }
  }

  function evidenceCount(p: PredictedPrompt): number {
    return p.artifacts?.scenario_ids?.length ?? 0;
  }

  async function adopt(id: string) {
    try {
      const intent = await invoke<string>("adopt_prediction", { predictionId: id });
      showToast(
        `Prediction adopted — ${intent}. Your agent's next prompt will use this package.`,
        "success",
        5000,
      );
    } catch (err) {
      const msg = typeof err === "string" ? err : "Couldn't adopt that prediction.";
      showToast(msg, "attention", 5000);
    }
  }
</script>

<QuietShell markState="active" stateLabel={`Pondering · ${predictions.length} active`} stateLabelTint="var(--vd-st-active)">
  <V1Kicker text="Likely next steps" color="var(--vd-st-active)" />
  <div class="gap-6"></div>
  <V1Headline text={predictions[0]?.spec.label ?? "Vaner is preparing options"} />

  <div class="rows">
    {#each visiblePredictions as p, i (p.id)}
      <div class="row" class:lead={i === 0}>
        <div class="row-body">
          <div class="row-label">{p.spec.label}</div>
          {#if p.ui_summary || p.spec.description}
            <div class="row-summary">{p.ui_summary ?? p.spec.description}</div>
          {/if}
          <div class="row-meta">
            <span>{typeLabel(p)}</span>
            <span class={`pill readiness-${p.run.readiness}`}>{p.run.readiness}</span>
            <span class="conf">{Math.round(p.spec.confidence * 100)}%</span>
            {#if evidenceCount(p) > 0}
              <span>{evidenceCount(p)} evidence</span>
            {/if}
          </div>
        </div>
        {#if i === 0}
          <V1PrimaryButton
            title="Adopt"
            tint="var(--vd-st-active)"
            disabled={!isAdoptable(p.run.readiness)}
            onclick={() => adopt(p.id)}
          />
        {:else}
          <V1GhostButton
            title="Adopt"
            disabled={!isAdoptable(p.run.readiness)}
            onclick={() => adopt(p.id)}
          />
        {/if}
      </div>
    {/each}
  </div>

  <PopoverContextBlock {context} compact />
  <PopoverQuickActions cockpitPrimary={false} tab="prepared" />

  {#snippet footer()}
    <PopoverFooter health="active" healthLabel={`Last update ${context.lastUpdateLabel}`} detailsTab="prepared" />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .rows {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 12px;
  }
  .row {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
  }
  .row.lead {
    background: var(--vd-bg-2);
  }
  .row-body { flex: 1 1 auto; min-width: 0; }
  .row-label {
    font-family: var(--vd-font);
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .row-summary {
    margin-top: 4px;
    color: var(--vd-fg-4);
    font-family: var(--vd-font);
    font-size: 11px;
    line-height: 1.35;
  }
  .row-meta {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
    font-family: var(--vd-font);
    font-size: 11px;
  }
  .pill {
    padding: 2px 7px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.06);
    color: var(--vd-fg-2);
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-size: 10px;
  }
  .readiness-ready { background: color-mix(in srgb, var(--vd-st-on) 18%, transparent); color: var(--vd-st-on); }
  .readiness-drafting { background: color-mix(in srgb, var(--vd-st-active) 18%, transparent); color: var(--vd-st-active); }
  .conf {
    color: var(--vd-fg-3);
    font-family: var(--vd-font-mono);
    font-variant-numeric: tabular-nums;
  }
</style>
