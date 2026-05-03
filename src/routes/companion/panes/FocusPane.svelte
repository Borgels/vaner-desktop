<script lang="ts">
  import { onDestroy, onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import { focusRoute, refreshFocusRoute, startFocusPolling, stopFocusPolling, updateFocusRoute } from "$lib/stores/focus.js";

  onMount(() => {
    startFocusPolling();
    void refreshFocusRoute();
  });
  onDestroy(() => stopFocusPolling());

  const route = $derived($focusRoute.effective_route);
  const workspace = $derived(route.workspace);
  const client = $derived(route.client);
  const backend = $derived(route.backend);
  const overrideActive = $derived(route.workspace_policy === "pinned" || route.workspace_policy === "work_here" || !!client?.preferred);
  const headline = $derived($focusRoute.explanation || "Vaner has not selected an active workspace yet.");
  const workspacePath = $derived(workspace?.canonical_path ?? "");
  const selectedResource = $derived(route.resource_mode ?? "balanced");
  const selectedDevice = $derived(route.device ?? "auto");
  const selectedModel = $derived(backend?.model || "Not set");

  async function chooseWorkspace(policy: "work_here" | "pinned") {
    const path = await invoke<string | null>("workspace_pick");
    if (path) await updateFocusRoute({ workspace_policy: policy, workspace_path: path });
  }
</script>

<section class="routing-pane">
  <header class="pane-head">
    <div>
      <VSectionLabel text="Routing" />
      <h1>{headline}</h1>
      <p>
        {#if overrideActive}
          Auto Focus is still available. This route is using your current override.
        {:else if workspace}
          Workspace selected automatically from the active wired client.
        {:else}
          Choose a workspace now, or keep Auto enabled and Vaner will start when a supported client becomes active.
        {/if}
      </p>
    </div>
    <div class="route-badge">{overrideActive ? "Override" : "Auto"}</div>
  </header>

  <section class="summary" aria-label="Current Vaner route">
    <div class="summary-row">
      <span>Workspace</span>
      <strong>{workspace?.display_name ?? "Auto"}</strong>
      {#if workspacePath}<small>{workspacePath}</small>{/if}
    </div>
    <div class="summary-row">
      <span>Client</span>
      <strong>{client?.display_name ?? "Auto"}</strong>
      <small>{client?.preferred ? "Preferred client" : client?.running ? "Running now" : "Selected automatically"}</small>
    </div>
    <div class="summary-row">
      <span>Compute</span>
      <strong>{selectedResource.replace("_", " ")} · {selectedDevice}</strong>
      <small>{backend?.name || "Backend"} · {selectedModel}</small>
    </div>
  </section>

  <section class="actions" aria-label="Route actions">
    {#if workspace}
      <button onclick={() => updateFocusRoute({ workspace_policy: "work_here", workspace_path: workspacePath })}>
        Use this workspace now
      </button>
      <button onclick={() => updateFocusRoute({ workspace_policy: "pinned", workspace_path: workspacePath })}>
        Make default
      </button>
    {:else}
      <button onclick={() => chooseWorkspace("work_here")}>Choose workspace</button>
    {/if}
    <button onclick={() => chooseWorkspace("work_here")}>Choose another workspace</button>
    {#if overrideActive}
      <button onclick={() => updateFocusRoute({ workspace_policy: "auto", client_id: null })}>Return to auto</button>
    {/if}
  </section>

  <section class="grid">
    <div class="panel">
      <VSectionLabel text="Workspace" />
      <button class:active={route.workspace_policy === "auto"} onclick={() => updateFocusRoute({ workspace_policy: "auto" })}>
        Auto
      </button>
      {#each $focusRoute.workspace_options as option (option.id)}
        <button
          class:active={option.selected}
          disabled={!option.canonical_path}
          onclick={() => updateFocusRoute({ workspace_policy: "work_here", workspace_path: option.canonical_path ?? undefined })}
        >
          {option.display_name}
          <small>{option.pinned ? "default" : option.paused ? "paused" : option.eligible ? "eligible" : "detected"}</small>
        </button>
      {/each}
    </div>

    <div class="panel">
      <VSectionLabel text="Client connection" />
      <button class:active={!client?.preferred} onclick={() => updateFocusRoute({ client_id: null })}>Auto</button>
      {#if $focusRoute.client_options.length}
        {#each $focusRoute.client_options as option (option.id)}
          <button class:active={option.selected} onclick={() => updateFocusRoute({ client_id: option.id })}>
            {option.display_name}
            <small>{option.running ? "running" : option.integration_state}</small>
          </button>
        {/each}
      {:else}
        <div class="empty">No eligible wired client for this workspace yet.</div>
      {/if}
    </div>

    <div class="panel">
      <VSectionLabel text="Compute and model" />
      {#each $focusRoute.hardware_options.resource_modes as mode (mode.id)}
        <button class:active={selectedResource === mode.id} onclick={() => updateFocusRoute({ resource_mode: mode.id })}>
          {mode.label}
        </button>
      {/each}
      <details>
        <summary>Advanced</summary>
        <label>
          Device
          <select onchange={(event) => updateFocusRoute({ compute_device: event.currentTarget.value })}>
            <option value="auto" selected={selectedDevice === "auto"}>Auto</option>
            <option value="cpu" selected={selectedDevice === "cpu"}>CPU</option>
            <option value="cuda" selected={selectedDevice === "cuda"}>CUDA</option>
            <option value="mps" selected={selectedDevice === "mps"}>Apple GPU</option>
          </select>
        </label>
      </details>
    </div>
  </section>

  <details class="details">
    <summary>Routing details</summary>
    {#if $focusRoute.diagnostics.why_not?.length}
      <div class="detail-list">
        {#each $focusRoute.diagnostics.why_not as reason}
          <div>{reason.message ?? reason.reason_code}</div>
        {/each}
      </div>
    {/if}
    <div class="detail-list">
      {#each $focusRoute.diagnostics.raw_detected_clients ?? [] as detected}
        <div>{detected.display_name} · {detected.integration_state}</div>
      {/each}
    </div>
  </details>
</section>

<style>
  .routing-pane {
    display: flex;
    flex-direction: column;
    gap: 18px;
  }
  .pane-head {
    display: flex;
    justify-content: space-between;
    gap: 18px;
    align-items: flex-start;
  }
  h1 {
    margin: 6px 0;
    font-size: 26px;
    letter-spacing: 0;
    line-height: 1.15;
  }
  p {
    margin: 0;
    color: var(--vd-fg-2);
    max-width: 680px;
    line-height: 1.5;
  }
  .route-badge {
    border: 0.5px solid var(--vd-hair);
    padding: 6px 10px;
    font-size: 12px;
    color: var(--vd-fg-2);
  }
  .summary {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    border: 0.5px solid var(--vd-hair);
    border-radius: 8px;
    overflow: hidden;
  }
  .summary-row {
    min-width: 0;
    padding: 14px;
    border-right: 0.5px solid var(--vd-hair);
    display: grid;
    gap: 5px;
  }
  .summary-row:last-child {
    border-right: 0;
  }
  span,
  small,
  .empty,
  summary {
    color: var(--vd-fg-3);
    font-size: 12px;
  }
  strong {
    color: var(--vd-fg-1);
    font-size: 15px;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .actions,
  .panel {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
  }
  .grid {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 16px;
  }
  .panel {
    align-content: flex-start;
    border-top: 0.5px solid var(--vd-hair);
    padding-top: 14px;
  }
  .panel :global(.section-label) {
    flex-basis: 100%;
  }
  button,
  select {
    min-height: 32px;
    border: 0.5px solid var(--vd-hair);
    border-radius: 6px;
    background: var(--vd-bg-1);
    color: var(--vd-fg-1);
    padding: 0 12px;
    font: inherit;
    font-size: 12px;
  }
  button.active {
    border-color: var(--vd-accent);
    background: var(--vd-bg-2);
  }
  button small {
    margin-left: 8px;
  }
  button:hover:not(:disabled) {
    background: var(--vd-bg-2);
  }
  details {
    flex-basis: 100%;
  }
  label {
    display: grid;
    gap: 6px;
    color: var(--vd-fg-3);
    font-size: 12px;
    margin-top: 10px;
  }
  .details {
    border-top: 0.5px solid var(--vd-hair);
    padding-top: 12px;
  }
  .detail-list {
    display: grid;
    gap: 6px;
    margin-top: 10px;
    color: var(--vd-fg-2);
    font-size: 12px;
  }
  @media (max-width: 860px) {
    .pane-head,
    .summary,
    .grid {
      grid-template-columns: 1fr;
      flex-direction: column;
    }
    .summary-row {
      border-right: 0;
      border-bottom: 0.5px solid var(--vd-hair);
    }
  }
</style>
