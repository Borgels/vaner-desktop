<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import MCPClientsPanel from "./MCPClientsPanel.svelte";
  import { rescan } from "$lib/stores/clients.js";

  type Tab = "clients" | "engine" | "telemetry";

  let activeTab: Tab = "clients";

  onMount(() => {
    // Trigger an initial detection scan when the route opens.
    rescan();
  });

  function close() {
    goto("/");
  }
</script>

<div class="prefs-shell">
  <header class="prefs-header">
    <button class="back" type="button" on:click={close} aria-label="Back to predictions">
      ←
    </button>
    <h1>Preferences</h1>
  </header>

  <div class="tabs" role="tablist" aria-label="Preferences sections">
    <button
      role="tab"
      aria-selected={activeTab === "clients"}
      class:active={activeTab === "clients"}
      type="button"
      on:click={() => (activeTab = "clients")}
    >
      MCP Clients
    </button>
    <button
      role="tab"
      aria-selected={activeTab === "engine"}
      class:active={activeTab === "engine"}
      type="button"
      on:click={() => (activeTab = "engine")}
      title="Engine settings — coming in 0.8.6"
      disabled
    >
      Engine
    </button>
    <button
      role="tab"
      aria-selected={activeTab === "telemetry"}
      class:active={activeTab === "telemetry"}
      type="button"
      on:click={() => (activeTab = "telemetry")}
      title="Telemetry settings — coming in 0.8.6"
      disabled
    >
      Telemetry
    </button>
  </div>

  <section class="prefs-body">
    {#if activeTab === "clients"}
      <MCPClientsPanel />
    {/if}
  </section>
</div>

<style>
  .prefs-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--vaner-bg-0, #111);
    color: var(--vaner-fg, #f2f2f2);
    font-family: system-ui, -apple-system, sans-serif;
  }
  .prefs-header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 14px 18px;
    border-bottom: 1px solid var(--vaner-hair, #2a2a2a);
  }
  .prefs-header h1 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }
  button.back {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    width: 28px;
    height: 28px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 14px;
  }
  button.back:hover {
    background: var(--vaner-hover, rgba(255, 255, 255, 0.05));
  }
  .tabs {
    display: flex;
    gap: 4px;
    padding: 8px 18px 0;
    border-bottom: 1px solid var(--vaner-hair, #2a2a2a);
  }
  .tabs button {
    background: transparent;
    border: none;
    color: var(--vaner-muted, #888);
    padding: 8px 14px;
    border-radius: 6px 6px 0 0;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
    border-bottom: 2px solid transparent;
  }
  .tabs button:hover:not(:disabled) {
    color: inherit;
  }
  .tabs button.active {
    color: inherit;
    border-bottom-color: var(--vaner-accent, #5eb2ff);
  }
  .tabs button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .prefs-body {
    flex: 1;
    overflow: auto;
    padding: 18px;
  }
</style>
