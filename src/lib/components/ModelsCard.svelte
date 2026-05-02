<!--
  Models card — Linux equivalent of the macOS app's Models pane,
  scoped to Ollama for v1. Shows installed models with a "Use" /
  "Remove" affordance, lets the user pull new ones (suggested
  shortcuts + a custom-name field), and surfaces an active pull as
  a sticky progress card with a Cancel button.

  Rendered inside PreferencesPane between Active setup and Silent
  hours so it's the first power-user toggle the user reaches.
-->
<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import {
    activePull,
    bootstrapOllamaListeners,
    cancelPull,
    disposeOllamaListeners,
    ollamaState,
    pullModel,
    refreshOllama,
    removeModel,
    useModel,
  } from "$lib/stores/ollama.js";
  import { showToast } from "$lib/stores/toast.js";
  import {
    loadModelRecommendation,
    type ModelsRecommendedPayload,
    type RecommendedModelEntry,
  } from "$lib/stores/setup.js";

  let customName = $state("");
  let busyAction = $state(false);
  let confirmRemove = $state<string | null>(null);
  let currentModel = $state("");
  let recommendation = $state<ModelsRecommendedPayload | null>(null);

  // Suggestions for the "Pull a new model" section come from the
  // daemon's hardware-aware recommender, not a hardcoded list. The
  // engine has the registry of which models fit which budget; the
  // desktop just displays whatever it returns. Filter to alternatives
  // that actually fit the user's hardware budget, plus the recommended
  // pick — no point offering a 32B coder model on a 4 GB VRAM laptop,
  // and equally no point offering a 2B fallback on a 96 GB workstation.
  const suggestedFromRegistry = $derived.by<RecommendedModelEntry[]>(() => {
    if (!recommendation) return [];
    const budgetGb = recommendation.budget?.effective_gb_q4 ?? null;
    const fits = (m: RecommendedModelEntry) => {
      if (budgetGb == null) return true;
      const need =
        m.recommended_effective_memory_gb ??
        m.min_effective_memory_gb ??
        m.min_effective_gb_q4 ??
        0;
      return need <= budgetGb;
    };
    const all: RecommendedModelEntry[] = [];
    if (recommendation.selected) all.push(recommendation.selected);
    for (const alt of recommendation.alternatives ?? []) {
      if (fits(alt) && !all.some((m) => modelKey(m) === modelKey(alt))) all.push(alt);
    }
    return all;
  });

  function modelKey(m: RecommendedModelEntry): string {
    return m.model_id ?? m.id ?? m.family ?? "";
  }
  function modelDisplayTitle(m: RecommendedModelEntry): string {
    return m.display_name ?? m.family ?? m.id ?? "Local model";
  }
  function modelDisplaySize(m: RecommendedModelEntry): string {
    const gb =
      m.recommended_effective_memory_gb ??
      m.min_effective_memory_gb ??
      m.min_effective_gb_q4;
    return gb ? `${gb} GB` : "";
  }

  // Read the live backend.model out of `vaner status --json`, which
  // the desktop already shells via the diagnostics_status Tauri
  // command. The status payload has `.backend.model`. Parse loosely
  // — older CLIs that don't surface backend show "" here, which the
  // card handles fine.
  async function refreshCurrentModel() {
    try {
      const status = (await invoke("diagnostics_status")) as { backend?: { model?: string } };
      currentModel = status?.backend?.model ?? "";
    } catch {
      currentModel = "";
    }
  }

  onMount(async () => {
    await bootstrapOllamaListeners();
    await Promise.all([
      refreshOllama(),
      refreshCurrentModel(),
      (async () => {
        recommendation = await loadModelRecommendation();
      })(),
    ]);
  });

  onDestroy(() => {
    void disposeOllamaListeners();
  });

  async function handleUse(name: string) {
    if (busyAction) return;
    busyAction = true;
    try {
      await useModel(name);
      showToast(`Vaner is now using ${name}.`, "success", 3000);
      await refreshCurrentModel();
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not switch model: ${err}`,
        "attention",
        4000,
      );
    } finally {
      busyAction = false;
    }
  }

  async function handleRemove(name: string) {
    if (busyAction) return;
    busyAction = true;
    try {
      await removeModel(name);
      showToast(`Removed ${name}.`, "info", 2500);
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not remove ${name}: ${err}`,
        "attention",
        4000,
      );
    } finally {
      confirmRemove = null;
      busyAction = false;
    }
  }

  async function handlePull(name: string) {
    if (busyAction || $activePull) return;
    const trimmed = name.trim();
    if (!trimmed) {
      showToast("Type a model name first.", "info", 2000);
      return;
    }
    busyAction = true;
    try {
      await pullModel(trimmed);
      // Pull is fire-and-forget; the active-pull store fills in via
      // the Tauri event listener.
      if (name === customName) customName = "";
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not start pull: ${err}`,
        "attention",
        4000,
      );
    } finally {
      busyAction = false;
    }
  }

  async function handleCancel() {
    try {
      await cancelPull();
      showToast("Pull cancelled.", "info", 2000);
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not cancel: ${err}`,
        "attention",
        3000,
      );
    }
  }

  function progressPercent(p: { fraction: number | null }): number | null {
    if (p.fraction == null) return null;
    return Math.round(p.fraction * 100);
  }
</script>

<div class="card">
  <div class="card-head"><span class="rail"></span><span>Models (Ollama)</span></div>

  {#if $activePull}
    {@const pull = $activePull}
    {@const pct = progressPercent(pull)}
    <div class="pull-progress" role="status" aria-live="polite">
      <div class="pull-head">
        <strong>Pulling <code>{pull.model}</code></strong>
        <V1GhostButton title="Cancel" destructive onclick={handleCancel} />
      </div>
      <div class="pull-bar" aria-hidden="true">
        <div class="pull-fill" style:width="{pct ?? 0}%"></div>
      </div>
      <span class="pull-status">
        {pull.status}
        {#if pct != null}· {pct}%{/if}
        {#if pull.completed_bytes != null && pull.total_bytes != null}
          · {(pull.completed_bytes / 1024 / 1024 / 1024).toFixed(2)} / {(pull.total_bytes / 1024 / 1024 / 1024).toFixed(2)} GiB
        {/if}
      </span>
    </div>
  {/if}

  {#if !$ollamaState.available}
    <p class="hint">{$ollamaState.detail || "Ollama is unreachable."}</p>
    <div class="actions">
      <V1GhostButton title="Retry" onclick={refreshOllama} />
    </div>
  {:else}
    {#if $ollamaState.models.length === 0}
      <p class="empty">No models pulled yet. Grab one below to get started.</p>
    {:else}
      <ul class="models">
        {#each $ollamaState.models as model (model.name)}
          {@const inUse = model.name === currentModel}
          <li class="model-row" class:in-use={inUse}>
            <div class="model-id">
              <code class="model-name">{model.name}</code>
              <span class="model-size">{model.size_display}</span>
              {#if inUse}<span class="badge">IN USE</span>{/if}
            </div>
            <div class="model-actions">
              {#if !inUse}
                <V1GhostButton title="Use" onclick={() => handleUse(model.name)} />
              {/if}
              <V1GhostButton
                title="Remove"
                destructive
                onclick={() => (confirmRemove = model.name)}
                disabled={inUse}
              />
            </div>
            {#if confirmRemove === model.name}
              <div class="confirm-inline">
                <span>Remove <code>{model.name}</code> from disk?</span>
                <V1GhostButton title="Cancel" onclick={() => (confirmRemove = null)} />
                <V1GhostButton title="Remove" destructive onclick={() => handleRemove(model.name)} />
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <div class="pull-section">
      <span class="section-label">Recommended for your hardware</span>
      <div class="suggested">
        {#each suggestedFromRegistry as s, i (modelKey(s))}
          {@const key = modelKey(s)}
          {@const installed = $ollamaState.models.some((m) => m.name === key)}
          {@const isTopPick = i === 0}
          <button
            type="button"
            class="suggest-row"
            class:top={isTopPick}
            disabled={busyAction || !!$activePull || installed}
            onclick={() => handlePull(key)}
          >
            <code class="suggest-name">{key}</code>
            <span class="suggest-title">{modelDisplayTitle(s)}</span>
            {#if modelDisplaySize(s)}
              <span class="suggest-size">{modelDisplaySize(s)}</span>
            {/if}
            {#if installed}
              <span class="badge muted">INSTALLED</span>
            {:else if isTopPick}
              <span class="badge top">RECOMMENDED</span>
            {/if}
          </button>
        {/each}
        {#if suggestedFromRegistry.length === 0}
          <p class="empty">Loading recommendations…</p>
        {/if}
      </div>
      <div class="custom-row">
        <input
          type="text"
          placeholder="custom-name:tag (e.g. llama3.2:3b)"
          bind:value={customName}
          disabled={busyAction || !!$activePull}
        />
        <V1PrimaryButton
          title="Pull"
          onclick={() => handlePull(customName)}
          tint="var(--vd-amber)"
        />
      </div>
    </div>
  {/if}
</div>

<style>
  .card {
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    padding: 18px 20px;
    margin-bottom: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    margin-bottom: 4px;
  }
  .card-head .rail {
    width: 2px;
    height: 14px;
    border-radius: 1px;
    background: var(--vd-amber);
    flex: 0 0 auto;
  }
  .empty {
    margin: 0;
    font-size: 12px;
    color: var(--vd-fg-3);
  }
  .actions {
    display: flex;
    gap: 6px;
  }

  .pull-progress {
    display: flex;
    flex-direction: column;
    gap: 8px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--vd-amber) 8%, var(--vd-bg-1));
    border: 0.5px solid color-mix(in srgb, var(--vd-amber) 35%, transparent);
    border-radius: var(--vd-r-chip);
  }
  .pull-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    font-size: 12.5px;
    color: var(--vd-fg-1);
  }
  .pull-bar {
    width: 100%;
    height: 4px;
    background: rgba(255, 255, 255, 0.08);
    border-radius: 2px;
    overflow: hidden;
  }
  .pull-fill {
    height: 100%;
    background: var(--vd-amber);
    transition: width 180ms ease-out;
  }
  .pull-status {
    font-family: var(--vd-font-mono);
    font-size: 10.5px;
    color: var(--vd-fg-3);
  }

  .models {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .model-row {
    display: grid;
    grid-template-columns: 1fr max-content;
    grid-template-rows: auto auto;
    column-gap: 12px;
    row-gap: 6px;
    align-items: center;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
  }
  .model-row.in-use {
    border-color: color-mix(in srgb, var(--vd-purple) 50%, transparent);
    background: color-mix(in srgb, var(--vd-purple) 6%, var(--vd-bg-1));
  }
  .model-id {
    display: flex;
    align-items: baseline;
    gap: 10px;
    flex-wrap: wrap;
  }
  .model-name {
    font-family: var(--vd-font-mono);
    font-size: 12.5px;
    color: var(--vd-fg-1);
  }
  .model-size {
    font-size: 11px;
    color: var(--vd-fg-3);
  }
  .badge {
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.1em;
    padding: 2px 6px;
    border-radius: 4px;
    background: color-mix(in srgb, var(--vd-purple) 18%, transparent);
    color: var(--vd-purple);
  }
  .badge.muted {
    background: rgba(255, 255, 255, 0.05);
    color: var(--vd-fg-3);
  }
  .model-actions {
    display: flex;
    gap: 6px;
  }
  .confirm-inline {
    grid-column: 1 / span 2;
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 10px;
    background: color-mix(in srgb, var(--vd-st-attention) 6%, transparent);
    border: 0.5px solid color-mix(in srgb, var(--vd-st-attention) 30%, transparent);
    border-radius: 6px;
    font-size: 11.5px;
    color: var(--vd-fg-2);
  }

  .pull-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 6px;
  }
  .section-label {
    font-size: 10.5px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--vd-fg-3);
  }
  .suggested {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .suggest-row {
    display: grid;
    grid-template-columns: max-content 1fr max-content;
    grid-template-rows: auto auto;
    column-gap: 12px;
    row-gap: 2px;
    align-items: baseline;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
    text-align: left;
    cursor: pointer;
    color: var(--vd-fg-1);
    font: inherit;
    transition: background 120ms ease;
  }
  .suggest-row:hover:not(:disabled) {
    background: var(--vd-bg-2);
  }
  .suggest-row:disabled { opacity: 0.5; cursor: not-allowed; }
  .suggest-name {
    grid-column: 1;
    grid-row: 1 / span 2;
    align-self: center;
    font-family: var(--vd-font-mono);
    font-size: 12px;
    color: var(--vd-fg-2);
  }
  .suggest-title {
    grid-column: 2;
    grid-row: 1;
    font-size: 12.5px;
    color: var(--vd-fg-1);
    font-weight: 500;
  }
  .hint {
    margin: 0;
    font-size: 11.5px;
    color: var(--vd-fg-3);
    line-height: 1.5;
  }
  .suggest-size {
    grid-column: 2;
    grid-row: 2;
    font-size: 10.5px;
    color: var(--vd-fg-3);
    font-family: var(--vd-font-mono);
  }
  .suggest-row .badge {
    grid-column: 3;
    grid-row: 1;
  }
  .suggest-row.top {
    border-color: color-mix(in srgb, var(--vd-amber) 50%, transparent);
    background: color-mix(in srgb, var(--vd-amber) 6%, var(--vd-bg-1));
  }
  .badge.top {
    background: color-mix(in srgb, var(--vd-amber) 18%, transparent);
    color: var(--vd-amber);
  }

  .custom-row {
    display: flex;
    gap: 6px;
    align-items: center;
  }
  .custom-row input {
    flex: 1 1 auto;
    background: rgba(255, 255, 255, 0.04);
    border: 0.5px solid var(--vd-hair);
    border-radius: 6px;
    padding: 7px 10px;
    color: var(--vd-fg-1);
    font-family: var(--vd-font-mono);
    font-size: 12px;
  }
  .custom-row input::placeholder { color: var(--vd-fg-3); }
  .custom-row input:focus { outline: 1px solid var(--vd-amber); border-color: transparent; }
</style>
