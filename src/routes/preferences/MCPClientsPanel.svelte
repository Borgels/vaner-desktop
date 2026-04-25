<script lang="ts">
  import { clients, install, installAll, rescan, uninstall } from "$lib/stores/clients.js";

  let pending: string | null = null;

  async function onInstall(id: string) {
    pending = id;
    try {
      await install(id);
    } finally {
      pending = null;
    }
  }

  async function onReinstall(id: string) {
    pending = id;
    try {
      await install(id, "", true);
    } finally {
      pending = null;
    }
  }

  async function onRemove(id: string) {
    pending = id;
    try {
      await uninstall(id);
    } finally {
      pending = null;
    }
  }

  async function onInstallAll() {
    pending = "__all__";
    try {
      await installAll();
    } finally {
      pending = null;
    }
  }

  async function onUpdateAll() {
    pending = "__all__";
    try {
      await installAll("", true);
    } finally {
      pending = null;
    }
  }

  async function onRefresh() {
    await rescan();
  }

  $: hasUnconfigured = $clients.clients.some((c) => c.detected && !c.configured);
  $: hasDrift = ($clients.doctor?.drift_count ?? 0) > 0;
</script>

<div class="panel">
  <header class="panel-header">
    <div>
      <h2>MCP Clients</h2>
      <p class="muted">
        Install Vaner into the MCP-aware editors and agents on this machine. Idempotent
        and backup-safe; user-configured non-Vaner servers are preserved.
      </p>
    </div>
    <div class="header-actions">
      <button type="button" on:click={onRefresh} disabled={$clients.isScanning}>Refresh</button>
      <button
        type="button"
        class="primary"
        on:click={onInstallAll}
        disabled={!hasUnconfigured || pending !== null}
      >
        Install for all
      </button>
    </div>
  </header>

  {#if hasDrift}
    <div class="drift-banner" role="alert">
      <strong>Vaner binary moved</strong> — {$clients.doctor?.drift_count} client(s) point at
      the old path. Reinstalling rewrites every configured client with the current
      <code>which vaner</code> path.
      <button type="button" class="primary" on:click={onUpdateAll} disabled={pending !== null}>
        Update All
      </button>
    </div>
  {/if}

  {#if $clients.lastError}
    <div class="error">{$clients.lastError}</div>
  {/if}

  {#if $clients.isScanning && $clients.clients.length === 0}
    <div class="empty">Scanning…</div>
  {:else if $clients.hasInitialScan && $clients.clients.length === 0}
    <div class="empty">No MCP clients detected on this machine.</div>
  {:else}
    <ul class="client-list" aria-labelledby="mcp-clients-heading">
      {#each $clients.clients as c (c.id)}
        {@const isPending = pending === c.id || pending === "__all__"}
        <li class="client-row">
          <span class="status status-{c.status}" aria-label={c.status}>
            {#if c.status === "configured"}
              ✓ Configured
            {:else if c.status === "installed"}
              · Detected
            {:else}
              ✗ Missing
            {/if}
          </span>
          <div class="client-meta">
            <div class="client-label">{c.label}</div>
            {#if c.config_path}
              <div class="client-path" title={c.config_path}>{c.config_path}</div>
            {:else}
              <div class="client-path muted">{c.detail}</div>
            {/if}
          </div>
          <div class="actions">
            {#if c.status === "configured"}
              <button type="button" disabled={isPending} on:click={() => onReinstall(c.id)}>
                Reinstall
              </button>
              <button
                type="button"
                class="danger"
                disabled={isPending}
                on:click={() => onRemove(c.id)}
              >
                Remove
              </button>
            {:else if c.status === "installed"}
              <button
                type="button"
                class="primary"
                disabled={isPending}
                on:click={() => onInstall(c.id)}
              >
                Install
              </button>
            {/if}
          </div>
        </li>
      {/each}
    </ul>
  {/if}
</div>

<style>
  .panel {
    max-width: 720px;
  }
  .panel-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 16px;
    margin-bottom: 16px;
  }
  .panel-header h2 {
    margin: 0 0 4px;
    font-size: 18px;
    font-weight: 600;
  }
  .panel-header p {
    margin: 0;
    font-size: 13px;
    line-height: 1.4;
  }
  .header-actions {
    display: flex;
    gap: 8px;
    flex-shrink: 0;
  }
  button {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    padding: 6px 12px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
  }
  button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  button.primary {
    background: var(--vaner-accent, #5eb2ff);
    border-color: var(--vaner-accent, #5eb2ff);
    color: var(--vaner-bg-0, #0a1520);
    font-weight: 600;
  }
  button.danger {
    color: var(--vaner-danger, #e88590);
    border-color: var(--vaner-danger, #e88590);
  }
  .muted {
    color: var(--vaner-muted, #888);
  }
  .empty {
    padding: 24px;
    text-align: center;
    color: var(--vaner-muted, #888);
  }
  .drift-banner {
    background: rgba(230, 182, 86, 0.1);
    border: 1px solid rgba(230, 182, 86, 0.4);
    border-radius: 6px;
    padding: 10px 14px;
    margin-bottom: 16px;
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 13px;
  }
  .drift-banner button {
    margin-left: auto;
  }
  .error {
    background: rgba(232, 133, 144, 0.08);
    border: 1px solid rgba(232, 133, 144, 0.3);
    border-radius: 6px;
    padding: 8px 12px;
    margin-bottom: 12px;
    font-size: 12px;
    color: var(--vaner-danger, #e88590);
  }
  .client-list {
    list-style: none;
    padding: 0;
    margin: 0;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    background: var(--vaner-bg-1, #1a1a1a);
  }
  .client-row {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 14px;
    border-bottom: 1px solid var(--vaner-hair, #2a2a2a);
  }
  .client-row:last-child {
    border-bottom: none;
  }
  .status {
    font-size: 12px;
    width: 110px;
    flex-shrink: 0;
  }
  .status-configured {
    color: var(--vaner-success, #6ed0a6);
  }
  .status-installed {
    color: var(--vaner-warn, #e6b656);
  }
  .status-missing {
    color: var(--vaner-muted, #888);
  }
  .client-meta {
    flex: 1;
    min-width: 0;
  }
  .client-label {
    font-weight: 600;
    font-size: 13px;
  }
  .client-path {
    font-family: ui-monospace, "SF Mono", Menlo, Consolas, monospace;
    font-size: 11px;
    color: var(--vaner-muted, #888);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions {
    display: flex;
    gap: 6px;
    flex-shrink: 0;
  }
</style>
