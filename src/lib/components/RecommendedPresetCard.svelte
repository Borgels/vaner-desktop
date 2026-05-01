<!--
  Recommended-preset card — the wizard's WS11 step that surfaces the
  daemon's hardware-driven model recommendation before apply.

  Three rendering modes:

   1. Full payload (Vaner v0.8.8+ with non-empty registry): show
      accelerator + working memory + selected model + alternatives.
   2. Budget-only (Vaner v0.8.8 with empty registry — dev checkout
      that hasn't run scripts/refresh_recommended_models.py): show
      accelerator + memory but explain Vaner will pick at runtime.
   3. CLI-unavailable / pre-0.8.8 daemon: the Tauri command's
      synthetic fallback comes back with generator='desktop-fallback:
      cli-unavailable'. Fall through to whatever the existing
      hardware_profile probe knows so the card still surfaces the
      tier + GPU rather than misleading "couldn't read this machine"
      copy.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import type { ModelsRecommendedPayload } from "$lib/stores/setup.js";
  import type { HardwareProfile } from "$lib/contract/setup-types.js";

  type Props = {
    payload: ModelsRecommendedPayload | null;
    loading: boolean;
    /** Hardware silhouette from the existing `hardware_profile` Tauri
     *  command — used as a fallback when the daemon CLI is too old
     *  to produce the budget payload. */
    hardware?: HardwareProfile | null;
  };
  const { payload, loading, hardware = null }: Props = $props();

  // Detect the synthetic-fallback shape produced by setup.rs when the
  // CLI doesn't support `models-recommended`. Distinguishes pre-0.8.8
  // daemons from genuine empty registries.
  const cliUnavailable = $derived(
    payload?.registry?.generator === "desktop-fallback:cli-unavailable",
  );

  function tierLabel(tier: string | undefined): string {
    switch (tier) {
      case "high_performance": return "High-performance system";
      case "capable": return "Capable system";
      case "light": return "Lightweight machine";
      default: return "Your machine";
    }
  }

  function gpuLabel(gpu: string | undefined): string | null {
    if (!gpu || gpu === "none") return null;
    return {
      nvidia: "NVIDIA GPU",
      amd: "AMD GPU",
      apple_silicon: "Apple Silicon",
      integrated: "integrated graphics",
    }[gpu] ?? null;
  }

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
  const alternatives = $derived(
    (payload?.alternatives ?? []).filter((m: { id: string }) => m.id !== selected?.id),
  );
  const registryEmpty = $derived((payload?.registry?.model_count ?? 0) === 0);

  // True when we have NO budget payload AND no fallback hardware — the
  // only case where we should claim hardware detection failed.
  const trueHardwareUnknown = $derived(!payload?.budget && !hardware);
</script>

<section class="card">
  <V1Kicker text="Recommended for your machine" color="var(--vd-amber)" />

  {#if loading}
    <div class="loading"><Spinner size={16} /><span>Reading hardware…</span></div>
  {:else if trueHardwareUnknown}
    <h2 class="title">We couldn't read this machine.</h2>
    <V1Body
      muted
      text="Vaner will pick a model from your local runtime when it starts. You can swap it later in Companion → Models."
    />
  {:else if !payload?.budget}
    <!--
      Budget unknown but we have a HardwareProfile from the existing
      `hardware_profile` Tauri command. Surface what we know rather
      than misleading the user that hardware detection failed.
    -->
    <h2 class="title">
      {tierLabel(hardware?.tier)}{#if gpuLabel(hardware?.gpu)} · {gpuLabel(hardware?.gpu)}{/if}
    </h2>
    {#if hardware?.ram_gb}
      <V1Body muted text={`${hardware.ram_gb} GB RAM${hardware.gpu_vram_gb ? ` · ${hardware.gpu_vram_gb} GB VRAM` : ""}`} />
    {/if}
    <p class="fallback">
      {#if cliUnavailable}
        Update Vaner to v0.8.8+ for hardware-tuned model recommendations.
        For now the engine will pick a model from your local runtime
        when it starts — swap later in Companion → Models.
      {:else}
        Vaner will pick a model from your local runtime when it starts.
        You can swap it later in Companion → Models.
      {/if}
    </p>
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
