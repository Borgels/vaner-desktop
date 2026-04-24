<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { predictions, startPredictionStream } from "$lib/stores/predictions.js";
  import { isAdoptable } from "$lib/contract/types.js";

  onMount(() => {
    startPredictionStream();
  });

  async function adopt(id: string) {
    try {
      await invoke("adopt_prediction", { predictionId: id });
    } catch (err) {
      console.error("[vaner] adopt failed:", err);
    }
  }
</script>

<main>
  <header class="shell-header">
    <div class="brand">
      <span class="wordmark">vaner</span><span class="wordmark-accent">_</span>
    </div>
    <div class="state-label">Pondering · {$predictions.length} active</div>
  </header>

  <section class="body">
    <div class="kicker">Vaner is thinking ahead</div>

    {#if $predictions.length === 0}
      <div class="empty">
        Nothing to show yet. If the daemon is running, new predictions
        will stream in here.
      </div>
    {/if}

    {#each $predictions as p (p.id)}
      {@const adoptable = isAdoptable(p.run.readiness)}
      <div class="row" class:dim={p.spec.hypothesis_type === "long_tail"}>
        <div class="row-body">
          <div class="row-label">{p.spec.label}</div>
          <div class="row-meta">
            <span class="pill readiness-{p.run.readiness}">{p.run.readiness}</span>
            <span class="confidence">{Math.round(p.spec.confidence * 100)}%</span>
          </div>
        </div>
        <button
          class="adopt-btn"
          class:primary={adoptable}
          disabled={!adoptable}
          onclick={() => adopt(p.id)}
        >
          Adopt
        </button>
      </div>
    {/each}
  </section>
</main>

<style>
  main {
    display: flex;
    flex-direction: column;
    min-height: 100vh;
  }

  .shell-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 18px 12px;
    border-bottom: 0.5px solid var(--vd-hair);
  }

  .brand {
    font-family: var(--vd-font);
    font-size: 17px;
    letter-spacing: 0.5px;
    color: var(--vd-fg-1);
  }

  .wordmark-accent {
    color: var(--vd-amber);
  }

  .state-label {
    font-size: 10.5px;
    font-weight: 500;
    letter-spacing: 0.6px;
    text-transform: uppercase;
    color: var(--vd-st-active);
  }

  .body {
    padding: 18px 16px 14px;
  }

  .kicker {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1.4px;
    text-transform: uppercase;
    color: var(--vd-st-active);
    margin-bottom: 10px;
  }

  .empty {
    font-size: 12.5px;
    color: var(--vd-fg-3);
    padding: 8px 0;
  }

  .row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 8px 10px;
    background: rgba(255, 255, 255, 0.02);
    border: 0.5px solid var(--vd-hair);
    border-radius: 7px;
    margin-bottom: 6px;
  }

  .row.dim .row-label {
    color: var(--vd-fg-3);
  }

  .row-body {
    flex: 1;
    min-width: 0;
  }

  .row-label {
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .row-meta {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }

  .pill {
    font-size: 10px;
    font-weight: 500;
    padding: 2px 6px;
    border-radius: 4px;
    background: rgba(255, 255, 255, 0.03);
    color: var(--vd-fg-4);
  }

  .pill.readiness-ready {
    color: var(--vd-st-on);
    background: rgba(116, 194, 156, 0.18);
  }
  .pill.readiness-drafting {
    color: var(--vd-st-active);
    background: rgba(111, 191, 244, 0.15);
  }
  .pill.readiness-evidence_gathering,
  .pill.readiness-grounding {
    color: var(--vd-purple-deep);
    background: rgba(178, 153, 209, 0.15);
  }
  .pill.readiness-stale {
    color: var(--vd-st-idle);
    background: var(--vd-hair);
  }

  .confidence {
    font-family: var(--vd-font-mono);
    font-size: 10.5px;
    color: var(--vd-fg-4);
  }

  .adopt-btn {
    font-family: var(--vd-font);
    font-size: 12px;
    padding: 7px 12px;
    border-radius: 6px;
    background: transparent;
    border: 0.5px solid var(--vd-line);
    color: var(--vd-fg-2);
    cursor: pointer;
  }

  .adopt-btn.primary {
    background: var(--vd-st-active);
    color: var(--vd-bg-0);
    border-color: transparent;
    font-weight: 500;
  }

  .adopt-btn:disabled {
    opacity: 0.4;
    cursor: default;
  }
</style>
