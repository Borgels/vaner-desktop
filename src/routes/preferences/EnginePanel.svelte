<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { setup, loadStatus, loadHardware, refresh } from "$lib/stores/setup.js";

  let refreshState: { busy: boolean; detail: string | null } = { busy: false, detail: null };

  onMount(() => {
    void loadStatus();
    void loadHardware();
  });

  async function reRunSetup() {
    await goto("/setup");
  }

  async function pingDaemon() {
    refreshState = { busy: true, detail: null };
    try {
      const result = await refresh();
      refreshState = { busy: false, detail: result.detail };
    } catch (err) {
      refreshState = { busy: false, detail: typeof err === "string" ? err : String(err) };
    }
  }

  $: status = $setup.status;
  $: bundle = $setup.bundle;
  $: hardware = $setup.hardware;
  $: completedAt = status?.setup?.completed_at ?? null;
</script>

<div class="panel">
  <header class="panel-header">
    <h2>Engine settings</h2>
    <p class="hint">
      Vaner chooses the local model setup for this machine. Use this
      page to check what is active or rerun setup.
    </p>
  </header>

  {#if $setup.lastError}
    <div class="error">{$setup.lastError}</div>
  {/if}

  <section class="card">
    <h3>Active setup</h3>
    {#if completedAt}
      {#if bundle}
        <p class="bundle-label">{bundle.label}</p>
        <p class="bundle-desc">{bundle.description}</p>
      {:else}
        <p class="empty">Vaner setup is complete.</p>
      {/if}
      {#if hardware}
        <p class="muted">
          {hardware.gpu_devices?.[0]?.name ?? hardware.gpu}
          {#if hardware.gpu_devices?.[0]?.memory_display_gb}
            · {hardware.gpu_devices[0].memory_display_gb} GB {hardware.gpu_devices[0].memory_kind === "unified" ? "unified memory" : "VRAM"}
          {:else if hardware.memory_display_gb}
            · {hardware.memory_display_gb} GB system memory
          {/if}
        </p>
      {/if}
    {:else}
      <p class="empty">Setup has not been completed yet.</p>
    {/if}
  </section>

  <section class="actions">
    <button type="button" class="primary" on:click={reRunSetup}>
      Re-run setup
    </button>
    <button type="button" on:click={pingDaemon} disabled={refreshState.busy}>
      {refreshState.busy ? "Refreshing…" : "Check engine"}
    </button>
  </section>
  {#if refreshState.detail}
    <p class="muted small">{refreshState.detail}</p>
  {/if}
</div>

<style>
  .panel {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .panel-header {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  h2 {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  h3 {
    margin: 0 0 8px;
    font-size: 13px;
    font-weight: 600;
  }
  .hint,
  .muted,
  .empty {
    margin: 0;
    color: var(--vaner-muted, #888);
    font-size: 12px;
    line-height: 1.45;
  }
  .small {
    font-size: 11px;
  }
  .card {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    padding: 14px;
    background: var(--vaner-card, #151515);
  }
  .bundle-label {
    margin: 0 0 4px;
    font-size: 13px;
    font-weight: 600;
  }
  .bundle-desc {
    margin: 0 0 8px;
    color: var(--vaner-muted, #888);
    font-size: 12px;
    line-height: 1.45;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  button {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 7px;
    background: transparent;
    color: inherit;
    padding: 7px 12px;
    cursor: pointer;
  }
  button.primary {
    background: var(--vd-amber, #d99b3d);
    border-color: transparent;
    color: #111;
  }
  button:disabled {
    opacity: 0.55;
    cursor: default;
  }
  .error {
    border: 1px solid rgba(255, 120, 120, 0.35);
    border-radius: 8px;
    padding: 10px 12px;
    color: #ffb3b3;
    background: rgba(255, 80, 80, 0.08);
    font-size: 12px;
  }
</style>
