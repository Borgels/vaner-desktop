<script lang="ts" context="module">
  function tierLabel(tier: string): string {
    switch (tier) {
      case "light":
        return "Light";
      case "capable":
        return "Capable";
      case "high_performance":
        return "High-performance";
      case "unknown":
        return "Unknown";
      default:
        return tier;
    }
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { setup, loadHardware, loadStatus } from "$lib/stores/setup.js";
  import { predictions, startPredictionStream } from "$lib/stores/predictions.js";

  // 0.8.6 Telemetry tab — numeric-only readout for now (no charts).
  // Surfaces:
  //   - HardwareProfile (from `vaner setup hardware --json`)
  //   - Active prediction count + per-source breakdown
  //   - Per-cycle wall-clock timing summary derived from the most
  //     recent predictions (minimum / median / max ETA bucket counts).
  // Charts and SSE timing histograms land in a follow-up.

  let isReloading = false;

  onMount(() => {
    void startPredictionStream();
    void loadStatus();
    void loadHardware();
  });

  async function reload() {
    isReloading = true;
    try {
      await Promise.all([loadHardware(), loadStatus()]);
    } finally {
      isReloading = false;
    }
  }

  $: hardware = $setup.hardware;
  $: bundle = $setup.bundle;
  $: predictionList = $predictions ?? [];
  $: activeCount = predictionList.length;
  $: predictionSources = countBy(predictionList, (p) => p.spec.source);
  $: predictionEtaBuckets = countBy(
    predictionList,
    (p) => p.eta_bucket ?? "unknown",
  );

  function countBy<T>(items: T[], key: (item: T) => string): [string, number][] {
    const counts = new Map<string, number>();
    for (const item of items) {
      const k = key(item);
      counts.set(k, (counts.get(k) ?? 0) + 1);
    }
    return Array.from(counts.entries()).sort((a, b) => b[1] - a[1]);
  }
</script>

<div class="panel">
  <header class="panel-header">
    <h2>Telemetry</h2>
    <p class="hint">
      Read-only view of the engine's hardware probe, the active
      prediction set, and the bundle's tuning knobs. No data leaves
      this machine.
    </p>
  </header>

  {#if $setup.lastError}
    <div class="error">{$setup.lastError}</div>
  {/if}

  <section class="card">
    <header class="card-header">
      <h3>Hardware profile</h3>
      <button type="button" on:click={reload} disabled={isReloading}>
        {isReloading ? "Refreshing…" : "Refresh"}
      </button>
    </header>
    {#if hardware}
      <table class="rows">
        <tbody>
          <tr>
            <th>Tier</th>
            <td><strong>{tierLabel(hardware.tier)}</strong></td>
          </tr>
          <tr>
            <th>OS</th>
            <td>{hardware.os}</td>
          </tr>
          <tr>
            <th>CPU class</th>
            <td>{hardware.cpu_class}</td>
          </tr>
          <tr>
            <th>RAM</th>
            <td>{hardware.ram_gb} GB</td>
          </tr>
          <tr>
            <th>GPU</th>
            <td>
              {hardware.gpu}
              {#if hardware.gpu_vram_gb !== null && hardware.gpu_vram_gb !== undefined}
                · {hardware.gpu_vram_gb} GB VRAM
              {/if}
            </td>
          </tr>
          <tr>
            <th>Battery</th>
            <td>{hardware.is_battery ? "yes" : "no"}</td>
          </tr>
          <tr>
            <th>Thermal-constrained</th>
            <td>{hardware.thermal_constrained ? "yes" : "no"}</td>
          </tr>
          <tr>
            <th>Detected runtimes</th>
            <td>
              {#if hardware.detected_runtimes.length === 0}
                <span class="muted">none</span>
              {:else}
                {hardware.detected_runtimes.join(", ")}
              {/if}
            </td>
          </tr>
          <tr>
            <th>Detected models</th>
            <td>
              {#if hardware.detected_models.length === 0}
                <span class="muted">none</span>
              {:else}
                <ul class="models">
                  {#each hardware.detected_models as row}
                    <li>
                      <code>{row[0]}</code> · {row[1]}
                      <span class="muted">({row[2]})</span>
                    </li>
                  {/each}
                </ul>
              {/if}
            </td>
          </tr>
        </tbody>
      </table>
    {:else if $setup.isLoading}
      <p class="empty">Probing hardware…</p>
    {:else}
      <p class="empty">No hardware profile yet — refresh to probe.</p>
    {/if}
  </section>

  <section class="card">
    <h3>Active predictions</h3>
    <table class="rows">
      <tbody>
        <tr>
          <th>In-flight count</th>
          <td><strong>{activeCount}</strong></td>
        </tr>
        {#if predictionSources.length > 0}
          <tr>
            <th>By source</th>
            <td>
              <ul class="kv-list">
                {#each predictionSources as [source, count]}
                  <li><code>{source}</code> · {count}</li>
                {/each}
              </ul>
            </td>
          </tr>
        {/if}
        {#if predictionEtaBuckets.length > 0}
          <tr>
            <th>By ETA bucket</th>
            <td>
              <ul class="kv-list">
                {#each predictionEtaBuckets as [bucket, count]}
                  <li><code>{bucket}</code> · {count}</li>
                {/each}
              </ul>
            </td>
          </tr>
        {/if}
      </tbody>
    </table>
    {#if activeCount === 0}
      <p class="empty">
        No active predictions. The engine streams a snapshot here
        whenever the daemon updates a frame.
      </p>
    {/if}
  </section>

  {#if bundle}
    <section class="card">
      <h3>Bundle tuning</h3>
      <table class="rows">
        <tbody>
          <tr><th>Bundle</th><td>{bundle.label}</td></tr>
          <tr>
            <th>Drafting aggressiveness</th>
            <td>{bundle.drafting_aggressiveness.toFixed(2)}</td>
          </tr>
          <tr>
            <th>Exploration ratio</th>
            <td>{bundle.exploration_ratio.toFixed(2)}</td>
          </tr>
          <tr>
            <th>Persistence strength</th>
            <td>{bundle.persistence_strength.toFixed(2)}</td>
          </tr>
          <tr>
            <th>Goal weighting</th>
            <td>{bundle.goal_weighting.toFixed(2)}</td>
          </tr>
        </tbody>
      </table>
    </section>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .panel-header h2 {
    margin: 0 0 4px;
    font-size: 14px;
    font-weight: 600;
  }
  .hint {
    margin: 0;
    color: var(--vaner-muted, #888);
    font-size: 12px;
    line-height: 1.45;
  }
  .card {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    padding: 14px 16px;
    background: var(--vaner-bg-1, #181818);
  }
  .card-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 12px;
  }
  .card-header button {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    padding: 4px 10px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 11px;
  }
  .card-header button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 13px;
    font-weight: 600;
  }
  table.rows {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  table.rows th {
    text-align: left;
    color: var(--vaner-muted, #888);
    font-weight: 500;
    padding: 4px 12px 4px 0;
    width: 45%;
    vertical-align: top;
  }
  table.rows td {
    padding: 4px 0;
    vertical-align: top;
  }
  ul.kv-list,
  ul.models {
    list-style: none;
    margin: 0;
    padding: 0;
  }
  ul.kv-list li,
  ul.models li {
    padding: 1px 0;
  }
  code {
    font-family: var(--vd-font-mono, ui-monospace, monospace);
    font-size: 11px;
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 4px;
    border-radius: 3px;
  }
  .empty {
    margin: 0;
    color: var(--vaner-muted, #888);
    font-size: 12px;
  }
  .muted {
    color: var(--vaner-muted, #888);
  }
  .error {
    border-left: 3px solid #ef4444;
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.08);
    color: #ef4444;
    font-size: 12px;
  }
</style>
