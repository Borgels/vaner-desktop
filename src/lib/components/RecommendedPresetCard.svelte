<!--
  Recommended-preset card — the wizard's WS11 step that surfaces the
  daemon's hardware-driven model recommendation before apply.

  Falls back gracefully when:
   - The daemon CLI predates v0.8.8 (Tauri command returns a synthetic
     empty payload — see setup.rs::models_recommended).
   - The daemon ships v0.8.8 but the registry is empty (dev checkout
     that has not run scripts/refresh_recommended_models.py).

  In both cases the card renders the budget (if known) plus a
  "Vaner will pick a model from your local runtime when it starts"
  line so the user knows the wizard isn't broken — just that the
  recommendation pipeline isn't loaded yet.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import type { ModelsRecommendedPayload } from "$lib/stores/setup.js";

  type Props = {
    payload: ModelsRecommendedPayload | null;
    loading: boolean;
  };
  const { payload, loading }: Props = $props();

  const accelerator = $derived(payload?.budget?.accelerator ?? null);
  const acceleratorLabel = $derived.by(() => {
    if (!accelerator) return null;
    return {
      nvidia: "NVIDIA GPU",
      amd: "AMD GPU",
      apple_silicon: "Apple Silicon",
      integrated: "integrated graphics",
      cpu_only: "CPU only",
      cluster: "multi-GPU / datacenter accelerator",
    }[accelerator];
  });

  const budgetGb = $derived(payload?.budget?.effective_gb_q4 ?? null);
  const gpuCount = $derived(payload?.budget?.gpu_count ?? null);
  const selected = $derived(payload?.selected ?? null);
  const alternatives = $derived((payload?.alternatives ?? []).filter((m) => m.id !== selected?.id));
  const registryEmpty = $derived((payload?.registry?.model_count ?? 0) === 0);
</script>

<section class="card">
  <V1Kicker text="Recommended for your machine" color="var(--vd-amber)" />

  {#if loading}
    <div class="loading"><Spinner size={16} /><span>Reading hardware…</span></div>
  {:else if !payload || !payload.budget}
    <h2 class="title">We couldn't read this machine.</h2>
    <V1Body
      muted
      text="Vaner will pick a model from your local runtime when it starts. You can swap it later in Companion → Models."
    />
  {:else}
    <h2 class="title">
      {#if acceleratorLabel}
        {acceleratorLabel}
        {#if budgetGb !== null}· {budgetGb.toFixed(0)} GB working memory{/if}
      {/if}
    </h2>

    {#if gpuCount && gpuCount > 1}
      <V1Body muted text={`${gpuCount} GPUs detected`} />
    {/if}

    {#if registryEmpty}
      <p class="fallback">
        Vaner will pick a model from your local runtime when it starts.
        You can swap it later in Companion → Models.
      </p>
    {:else if selected}
      <div class="model">
        <span class="family">{selected.family}</span>
        <span class="params">≈ {selected.params_b.toFixed(0)}B parameters</span>
        <span class="id">{selected.id}</span>
      </div>
      {#if alternatives.length > 0}
        <details class="alts">
          <summary>Alternatives ({alternatives.length})</summary>
          <ul>
            {#each alternatives as alt (alt.id)}
              <li>
                <span class="alt-family">{alt.family}</span>
                <span class="alt-id">{alt.id}</span>
                <span class="alt-params">{alt.params_b.toFixed(0)}B</span>
              </li>
            {/each}
          </ul>
        </details>
      {/if}
    {:else}
      <p class="fallback">
        No model in our registry fits this machine yet. Vaner will fall
        back to whatever's loaded in your local runtime.
      </p>
    {/if}

    {#if payload.budget.notes && payload.budget.notes.length > 0}
      <ul class="notes">
        {#each payload.budget.notes as note (note)}
          <li>{note}</li>
        {/each}
      </ul>
    {/if}
  {/if}
</section>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 18px 22px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    color: var(--vd-fg-1);
  }
  .loading {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--vd-fg-2);
  }
  .title {
    font-family: var(--vd-font);
    font-size: 18px;
    font-weight: 500;
    letter-spacing: -0.15px;
    margin: 0;
    color: var(--vd-fg-1);
  }
  .fallback {
    font-size: 13px;
    line-height: 1.5;
    color: var(--vd-fg-2);
    margin: 0;
  }
  .model {
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 8px 12px;
    margin-top: 4px;
  }
  .family {
    font-size: 14px;
    font-weight: 500;
    color: var(--vd-fg-1);
  }
  .params {
    font-size: 12px;
    color: var(--vd-fg-2);
  }
  .id {
    font-family: var(--vd-mono);
    font-size: 11px;
    color: var(--vd-fg-3);
  }
  .alts {
    margin-top: 6px;
    font-size: 12px;
    color: var(--vd-fg-2);
  }
  .alts summary {
    cursor: pointer;
    user-select: none;
  }
  .alts ul {
    margin: 6px 0 0;
    padding-left: 14px;
    list-style: disc;
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .alt-family { color: var(--vd-fg-2); margin-right: 8px; }
  .alt-id { font-family: var(--vd-mono); font-size: 11px; color: var(--vd-fg-3); margin-right: 8px; }
  .alt-params { font-size: 11px; color: var(--vd-fg-3); }
  .notes {
    margin: 6px 0 0;
    padding-left: 14px;
    list-style: disc;
    font-size: 11px;
    color: var(--vd-fg-3);
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
</style>
