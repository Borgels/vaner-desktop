<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    setup,
    setupMode,
    loadStatus,
    loadHardware,
    refresh,
  } from "$lib/stores/setup.js";
  import type {
    BackgroundPosture,
    CloudPosture,
    ComputePosture,
    Priority,
    WorkStyle,
  } from "$lib/contract/setup-types.js";

  // The five Simple-Mode questions are surfaced read-only here. The
  // `/setup` wizard is the canonical place to *change* them; the
  // Engine tab just shows the current state + a button into the wizard.
  const WORK_STYLE_LABELS: Record<WorkStyle, string> = {
    writing: "Writing",
    research: "Research",
    planning: "Planning",
    support: "Support",
    learning: "Learning",
    coding: "Coding",
    general: "General",
    mixed: "Mixed",
    unsure: "Unsure",
  };

  const PRIORITY_LABELS: Record<Priority, string> = {
    balanced: "Balanced",
    speed: "Speed",
    quality: "Quality",
    privacy: "Privacy",
    cost: "Cost",
    low_resource: "Low-resource",
  };

  const COMPUTE_LABELS: Record<ComputePosture, string> = {
    light: "Light",
    balanced: "Balanced",
    available_power: "Available power",
  };

  const CLOUD_LABELS: Record<CloudPosture, string> = {
    local_only: "Local only",
    ask_first: "Ask first",
    hybrid_when_worth_it: "Hybrid (when worth it)",
    best_available: "Best available",
  };

  const BACKGROUND_LABELS: Record<BackgroundPosture, string> = {
    minimal: "Minimal",
    normal: "Normal",
    idle_more: "Idle-more",
    deep_run_aggressive: "Deep-Run aggressive",
  };

  let refreshState: { busy: boolean; detail: string | null } = { busy: false, detail: null };

  onMount(() => {
    void loadStatus();
    void loadHardware();
  });

  async function reRunWizard() {
    await goto("/setup");
  }

  async function pingDaemon() {
    refreshState = { busy: true, detail: null };
    try {
      const result = await refresh();
      refreshState = { busy: false, detail: result.detail };
    } catch (err) {
      refreshState = {
        busy: false,
        detail: typeof err === "string" ? err : String(err),
      };
    }
  }

  $: status = $setup.status;
  $: bundle = $setup.bundle;
  $: setupSection = status?.setup ?? {};
  $: workStyles = (setupSection.work_styles ?? []) as WorkStyle[];
  $: priority = (setupSection.priority ?? null) as Priority | null;
  $: computePosture = (setupSection.compute_posture ?? null) as ComputePosture | null;
  $: cloudPosture = (setupSection.cloud_posture ?? null) as CloudPosture | null;
  $: backgroundPosture = (setupSection.background_posture ?? null) as BackgroundPosture | null;
  $: completedAt = setupSection.completed_at ?? null;
  $: overrides = (status?.applied_policy?.overrides_applied ?? []) as string[];
  $: cloudWideningWarning = overrides.find((row) => row.startsWith("WIDENS_CLOUD_POSTURE"));
</script>

<div class="panel">
  <header class="panel-header">
    <h2>Engine settings</h2>
    <p class="hint">
      Vaner's engine knobs are derived from a policy bundle that
      reflects your work style, priority, and posture choices. Pick
      Simple to use Vaner's defaults; pick Advanced to override.
    </p>
  </header>

  <div class="mode-toggle" role="radiogroup" aria-label="Setup mode">
    <button
      type="button"
      role="radio"
      aria-checked={$setupMode === "simple"}
      class:active={$setupMode === "simple"}
      on:click={() => setupMode.set("simple")}
    >
      Simple
    </button>
    <button
      type="button"
      role="radio"
      aria-checked={$setupMode === "advanced"}
      class:active={$setupMode === "advanced"}
      on:click={() => setupMode.set("advanced")}
    >
      Advanced
    </button>
  </div>

  {#if $setup.lastError}
    <div class="error">{$setup.lastError}</div>
  {/if}

  {#if $setupMode === "simple"}
    <section class="card">
      <h3>Your answers</h3>
      {#if !completedAt}
        <p class="empty">
          Setup hasn't been completed yet. Run the wizard to choose a
          policy bundle that matches your machine and priorities.
        </p>
      {:else}
        <dl class="kv">
          <dt>Work styles</dt>
          <dd>
            {#if workStyles.length === 0}
              <span class="muted">none selected</span>
            {:else}
              <div class="chips">
                {#each workStyles as style}
                  <span class="chip">{WORK_STYLE_LABELS[style] ?? style}</span>
                {/each}
              </div>
            {/if}
          </dd>
          <dt>Priority</dt>
          <dd>{priority ? (PRIORITY_LABELS[priority] ?? priority) : "—"}</dd>
          <dt>Compute posture</dt>
          <dd>{computePosture ? (COMPUTE_LABELS[computePosture] ?? computePosture) : "—"}</dd>
          <dt>Cloud posture</dt>
          <dd>{cloudPosture ? (CLOUD_LABELS[cloudPosture] ?? cloudPosture) : "—"}</dd>
          <dt>Background pondering</dt>
          <dd>
            {backgroundPosture
              ? (BACKGROUND_LABELS[backgroundPosture] ?? backgroundPosture)
              : "—"}
          </dd>
        </dl>
      {/if}
    </section>

    <section class="card">
      <h3>Selected bundle</h3>
      {#if bundle}
        <p class="bundle-label">{bundle.label}</p>
        <p class="bundle-desc">{bundle.description}</p>
        <details>
          <summary>Why this bundle?</summary>
          <ul class="reasons">
            {#each (status?.applied_policy?.overrides_applied ?? []) as reason}
              <li>{reason}</li>
            {/each}
            {#if (status?.applied_policy?.overrides_applied ?? []).length === 0}
              <li class="muted">No overrides recorded — bundle defaults apply as-is.</li>
            {/if}
          </ul>
        </details>
        {#if cloudWideningWarning}
          <p class="warning">
            ⚠ Selecting this bundle widened your cloud posture. Re-run
            the wizard if that wasn't intentional.
          </p>
        {/if}
      {:else}
        <p class="empty">No bundle selected — defaults apply.</p>
      {/if}
    </section>

    <section class="actions">
      <button type="button" class="primary" on:click={reRunWizard}>
        Re-run setup wizard
      </button>
      <button
        type="button"
        on:click={pingDaemon}
        disabled={refreshState.busy}
      >
        {refreshState.busy ? "Refreshing…" : "Ask daemon to reload"}
      </button>
    </section>
    {#if refreshState.detail}
      <p class="muted small">{refreshState.detail}</p>
    {/if}
  {:else}
    <!-- Advanced mode: read-only echoes of the engine knobs the bundle controls,
         each with the "Switch to Simple to manage" affordance. The granular
         knob editor is a separate follow-up; for 0.8.6 the Engine tab gives
         users a transparency view + the wizard exit. The free-form TOML edit
         path lives behind `vaner setup advanced`. -->
    <section class="card">
      <h3>Bundle-managed knobs</h3>
      <p class="hint">
        The selected bundle controls the values below. To override a
        single knob, edit <code>.vaner/config.toml</code> directly via
        <code>vaner setup advanced</code>. Switch back to Simple to
        re-pick a bundle.
      </p>
      {#if bundle}
        <table class="advanced">
          <tbody>
            <tr>
              <th>Local/cloud posture</th>
              <td><code>{bundle.local_cloud_posture}</code></td>
            </tr>
            <tr>
              <th>Runtime profile</th>
              <td><code>{bundle.runtime_profile}</code></td>
            </tr>
            <tr>
              <th>Spend profile</th>
              <td><code>{bundle.spend_profile}</code></td>
            </tr>
            <tr>
              <th>Latency profile</th>
              <td><code>{bundle.latency_profile}</code></td>
            </tr>
            <tr>
              <th>Privacy profile</th>
              <td><code>{bundle.privacy_profile}</code></td>
            </tr>
            <tr>
              <th>Drafting aggressiveness</th>
              <td><code>{bundle.drafting_aggressiveness.toFixed(2)}</code></td>
            </tr>
            <tr>
              <th>Exploration ratio</th>
              <td><code>{bundle.exploration_ratio.toFixed(2)}</code></td>
            </tr>
            <tr>
              <th>Persistence strength</th>
              <td><code>{bundle.persistence_strength.toFixed(2)}</code></td>
            </tr>
            <tr>
              <th>Goal weighting</th>
              <td><code>{bundle.goal_weighting.toFixed(2)}</code></td>
            </tr>
            <tr>
              <th>Context injection (default)</th>
              <td><code>{bundle.context_injection_default}</code></td>
            </tr>
            <tr>
              <th>Deep-Run preset</th>
              <td><code>{bundle.deep_run_profile}</code></td>
            </tr>
          </tbody>
        </table>
        <p class="muted small">
          Switch to Advanced is for transparency only in 0.8.6. A full
          per-knob editor lands in 0.8.7.
        </p>
      {:else}
        <p class="empty">No bundle selected — run the wizard first.</p>
      {/if}
    </section>
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
  .hint {
    margin: 0;
    color: var(--vaner-muted, #888);
    font-size: 12px;
    line-height: 1.45;
  }
  .mode-toggle {
    display: inline-flex;
    align-self: flex-start;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 6px;
    overflow: hidden;
  }
  .mode-toggle button {
    background: transparent;
    border: none;
    color: var(--vaner-muted, #888);
    padding: 6px 14px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
  }
  .mode-toggle button.active {
    background: var(--vaner-accent, #5eb2ff);
    color: var(--vaner-bg-0, #111);
  }
  .card {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    padding: 14px 16px;
    background: var(--vaner-bg-1, #181818);
  }
  .empty {
    margin: 0;
    color: var(--vaner-muted, #888);
    font-size: 12px;
  }
  .kv {
    margin: 0;
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: 18px;
    row-gap: 8px;
    font-size: 12px;
  }
  .kv dt {
    color: var(--vaner-muted, #888);
    font-weight: 500;
  }
  .kv dd {
    margin: 0;
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .chip {
    background: var(--vaner-bg-2, #222);
    padding: 2px 8px;
    border-radius: 999px;
    font-size: 11px;
  }
  .bundle-label {
    margin: 0 0 4px;
    font-weight: 600;
    font-size: 13px;
  }
  .bundle-desc {
    margin: 0 0 10px;
    color: var(--vaner-muted, #888);
    font-size: 12px;
    line-height: 1.45;
  }
  details summary {
    cursor: pointer;
    font-size: 12px;
    color: var(--vaner-accent, #5eb2ff);
  }
  .reasons {
    margin: 8px 0 0 18px;
    padding: 0;
    color: var(--vaner-fg, #f2f2f2);
    font-size: 12px;
    line-height: 1.5;
  }
  .reasons li {
    margin-bottom: 2px;
  }
  .warning {
    margin: 10px 0 0;
    padding: 8px 10px;
    border-left: 3px solid #f5a524;
    background: rgba(245, 165, 36, 0.08);
    color: #f5a524;
    font-size: 12px;
  }
  .actions {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .actions button {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    padding: 7px 14px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 500;
  }
  .actions button.primary {
    background: var(--vaner-accent, #5eb2ff);
    border-color: var(--vaner-accent, #5eb2ff);
    color: var(--vaner-bg-0, #111);
  }
  .actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .error {
    border-left: 3px solid #ef4444;
    padding: 6px 10px;
    background: rgba(239, 68, 68, 0.08);
    color: #ef4444;
    font-size: 12px;
  }
  .muted {
    color: var(--vaner-muted, #888);
  }
  .small {
    font-size: 11px;
  }
  .advanced {
    width: 100%;
    border-collapse: collapse;
    font-size: 12px;
  }
  .advanced th {
    text-align: left;
    color: var(--vaner-muted, #888);
    font-weight: 500;
    padding: 4px 12px 4px 0;
    width: 50%;
  }
  .advanced td {
    padding: 4px 0;
  }
  code {
    font-family: var(--vd-font-mono, ui-monospace, monospace);
    font-size: 11px;
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 4px;
    border-radius: 3px;
  }
</style>
