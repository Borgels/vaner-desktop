<!--
  WizardVerificationPanel — final-slide verification per the four-layer
  leverage stack (see docs.vaner.ai/integrations/client-capabilities).

  Why this exists
  ---------------
  Wiring Vaner as an MCP server is the floor. Without the per-client
  primer (and skills/plugins where applicable), the agent often won't
  call vaner.* tools even when MCP is wired. This panel reports
  per-layer status per detected client so the user sees which clients
  are "ready" vs "wired-mcp-only" — the latter being the failure mode
  the agent is most likely to ignore Vaner under.

  Data
  ----
  Pulls from the new `clients_verify` Tauri command (src-tauri/src/
  clients.rs) which shells out to `vaner clients verify --format json`.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import { showToast } from "$lib/stores/toast.js";

  type LayerStatus = {
    applicable: boolean;
    wired: boolean;
    path: string | null;
    detail?: string;
  };

  type ClientLayers = {
    mcp: LayerStatus;
    primer: LayerStatus;
    skill: LayerStatus;
    plugin: LayerStatus;
  };

  type ClientVerification = {
    client_id: string;
    label: string;
    detected: boolean;
    overall: "ready" | "wired-mcp-only" | "partial" | "missing" | "not-detected";
    layers: ClientLayers;
  };

  type Props = {
    repoRoot: string;
    /** Callback to retry the install for a single client when its row is in
     *  ``partial`` or ``missing`` state. The wizard wires this to
     *  ``clients_install`` for the matching id. */
    onRepair?: (clientId: string) => void | Promise<void>;
  };

  const { repoRoot, onRepair }: Props = $props();

  let results = $state<ClientVerification[]>([]);
  let loading = $state(true);
  let error = $state<string | null>(null);
  let showAll = $state(false);
  /** Per-row spinner flag while a single Repair is in flight, so
   *  clicking Install on Cursor doesn't grey out every other card.
   *  Pre-fix the panel reloaded the entire list — visually noisy and
   *  encouraged the user to think they had to wait for every client. */
  let busy = $state<Record<string, boolean>>({});

  /** A leverage-aware verification phrase — exercises both a basic
   *  tool call and the prepared-work surface in a single round-trip
   *  the user can paste into any wired client. Tests *leverage*, not
   *  just plumbing. */
  const VERIFICATION_PHRASE =
    "Without searching the codebase first, check if Vaner has prepared anything " +
    "relevant to a recent change in this repo, and tell me which vaner.* tool you used.";

  async function loadVerification() {
    loading = true;
    error = null;
    try {
      const fetched = await invoke<ClientVerification[]>("clients_verify", {
        repoRoot,
      });
      results = fetched;
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    void loadVerification();
  });

  const detectedRows = $derived(
    results.filter((r) => r.detected),
  );
  const undetectedCount = $derived(
    results.filter((r) => !r.detected).length,
  );

  function statusLabel(overall: ClientVerification["overall"]): string {
    switch (overall) {
      case "ready":
        return "Ready";
      case "wired-mcp-only":
        return "Wired, no primer";
      case "partial":
        return "Partial install";
      case "missing":
        return "Not wired";
      case "not-detected":
        return "Not installed";
    }
  }

  function statusTint(overall: ClientVerification["overall"]): string {
    switch (overall) {
      case "ready":
        return "var(--vd-st-on, #6cc76c)";
      case "wired-mcp-only":
      case "partial":
        return "var(--vd-amber, #e6b656)";
      case "missing":
        return "var(--vd-st-attention, #d27c7c)";
      case "not-detected":
        return "var(--vd-fg-3, #9a9aa2)";
    }
  }

  function layerCellLabel(layer: LayerStatus): string {
    if (!layer.applicable) return "—";
    return layer.wired ? "✓" : "✗";
  }

  function layerCellTint(layer: LayerStatus): string {
    if (!layer.applicable) return "var(--vd-fg-4, #6a6a72)";
    return layer.wired ? "var(--vd-st-on, #6cc76c)" : "var(--vd-st-attention, #d27c7c)";
  }

  async function copyVerificationPhrase() {
    try {
      await navigator.clipboard.writeText(VERIFICATION_PHRASE);
      showToast("Verification phrase copied", "success", 2200);
    } catch {
      showToast("Could not access clipboard", "attention", 2500);
    }
  }

  /** Re-verify a single client without disturbing the rest. The full
   *  `loadVerification()` walks every supported client; that's
   *  expensive (8+ subprocesses) and visually trashes the panel
   *  whenever the user clicks Install on one row. Refresh just the
   *  one. */
  async function reverifyOne(clientId: string) {
    try {
      const fetched = await invoke<ClientVerification[]>("clients_verify", {
        repoRoot,
      });
      // The CLI returns the full set; pluck just the one row we care
      // about and patch it into our existing array. Keeps the rest of
      // the rows visually stable even if their underlying state has
      // drifted (it'll catch up on next full reload).
      const next = fetched.find((r) => r.client_id === clientId);
      if (next) {
        results = results.map((r) => (r.client_id === clientId ? next : r));
      }
    } catch (err) {
      // Fall back to a full refresh only when the targeted reverify
      // failed; the user still gets *some* update.
      await loadVerification();
      throw err;
    }
  }

  async function handleRepair(clientId: string) {
    if (!onRepair) return;
    busy = { ...busy, [clientId]: true };
    try {
      await onRepair(clientId);
      await reverifyOne(clientId);
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not finish wiring ${clientId}`,
        "attention",
        3500,
      );
    } finally {
      busy = { ...busy, [clientId]: false };
    }
  }
</script>

<section class="verify" aria-labelledby="verify-heading">
  <V1Kicker text="Agent integration" color="var(--vd-fg-3)" />
  <h3 id="verify-heading" class="title">How deeply Vaner is wired into each agent</h3>
  <V1Body
    muted
    text="Vaner installs in up to four layers per client (see docs.vaner.ai/integrations/client-capabilities). MCP wiring alone often isn't enough — the agent needs the primer to know when to call Vaner."
  />

  {#if loading}
    <div class="loading"><Spinner size={16} /><span>Checking each client…</span></div>
  {:else if error}
    <p class="error" role="alert">Could not verify client status: {error}</p>
    <V1GhostButton title="Retry" onclick={() => void loadVerification()} />
  {:else if detectedRows.length === 0}
    <p class="muted">
      No supported AI clients detected on this machine. You can wire Vaner
      into Claude Code, Cursor, Zed, VS Code, Codex CLI, Cline, Continue,
      Windsurf, Roo Code, or Claude Desktop later via <code>vaner clients install</code>.
    </p>
  {:else}
    <ul class="rows" role="list">
      {#each detectedRows as r (r.client_id)}
        <li class="row" data-status={r.overall}>
          <div class="row-head">
            <span class="row-label">{r.label}</span>
            <span class="status" style="color: {statusTint(r.overall)};">
              {statusLabel(r.overall)}
            </span>
          </div>
          <div class="layers" aria-label="Layer status">
            <span class="layer" title={r.layers.mcp.detail || "MCP server"}>
              <span class="layer-tag">MCP</span>
              <span class="layer-mark" style="color: {layerCellTint(r.layers.mcp)};">
                {layerCellLabel(r.layers.mcp)}
              </span>
            </span>
            <span class="layer" title={r.layers.primer.detail || "Primer (rules file)"}>
              <span class="layer-tag">Primer</span>
              <span class="layer-mark" style="color: {layerCellTint(r.layers.primer)};">
                {layerCellLabel(r.layers.primer)}
              </span>
            </span>
            <span class="layer" title={r.layers.skill.detail || "Skill (vaner-feedback)"}>
              <span class="layer-tag">Skill</span>
              <span class="layer-mark" style="color: {layerCellTint(r.layers.skill)};">
                {layerCellLabel(r.layers.skill)}
              </span>
            </span>
            <span class="layer" title={r.layers.plugin.detail || "Plugin"}>
              <span class="layer-tag">Plugin</span>
              <span class="layer-mark" style="color: {layerCellTint(r.layers.plugin)};">
                {layerCellLabel(r.layers.plugin)}
              </span>
            </span>
          </div>
          {#if r.overall === "wired-mcp-only" || r.overall === "partial" || r.overall === "missing"}
            <div class="row-actions">
              <V1GhostButton
                title={busy[r.client_id]
                  ? "Wiring…"
                  : r.overall === "missing"
                    ? "Install"
                    : "Finish wiring"}
                disabled={busy[r.client_id] === true}
                onclick={() => void handleRepair(r.client_id)}
              />
            </div>
          {/if}
        </li>
      {/each}
    </ul>

    <div class="verify-phrase">
      <p class="phrase-label">Verify in your agent — paste this:</p>
      <pre class="phrase">{VERIFICATION_PHRASE}</pre>
      <V1GhostButton title="Copy" onclick={copyVerificationPhrase} />
      <p class="phrase-hint">
        If the agent answers and names a <code>vaner.*</code> tool, the leverage
        stack is doing its job. If it ignores Vaner and falls back to grepping
        your repo, the primer is missing or being ignored.
      </p>
    </div>

    {#if undetectedCount > 0}
      <button
        type="button"
        class="show-all"
        onclick={() => (showAll = !showAll)}
        aria-expanded={showAll}
      >
        {showAll ? "Hide" : "Show"} {undetectedCount} undetected client{undetectedCount === 1 ? "" : "s"}
      </button>
      {#if showAll}
        <ul class="rows undetected" role="list">
          {#each results.filter((r) => !r.detected) as r (r.client_id)}
            <li class="row" data-status={r.overall}>
              <div class="row-head">
                <span class="row-label">{r.label}</span>
                <span class="status" style="color: {statusTint(r.overall)};">
                  {statusLabel(r.overall)}
                </span>
              </div>
            </li>
          {/each}
        </ul>
      {/if}
    {/if}
  {/if}
</section>

<style>
  .verify {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 14px 16px;
    background: var(--vd-bg-1, #18181c);
    border: 0.5px solid var(--vd-line, #2a2a30);
    border-radius: var(--vd-r-card, 6px);
    margin-top: 14px;
    text-align: left;
    color: var(--vd-fg-1, #f0f0f0);
  }
  .title {
    margin: 0;
    font-size: 14px;
    font-weight: 600;
  }
  .loading {
    display: flex;
    align-items: center;
    gap: 8px;
    color: var(--vd-fg-2, #d0d0d6);
    font-size: 13px;
  }
  .error {
    margin: 0;
    font-size: 12px;
    color: var(--vd-st-attention, #d27c7c);
  }
  .muted {
    margin: 0;
    font-size: 12px;
    color: var(--vd-fg-3, #9a9aa2);
  }
  .rows {
    list-style: none;
    margin: 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 8px 10px;
    border: 0.5px solid var(--vd-line, #2a2a30);
    border-radius: 4px;
    background: var(--vd-bg-2, #1d1d22);
  }
  .row[data-status="ready"] {
    border-color: color-mix(in srgb, var(--vd-st-on, #6cc76c) 30%, var(--vd-line, #2a2a30));
  }
  .row[data-status="wired-mcp-only"],
  .row[data-status="partial"] {
    border-color: color-mix(in srgb, var(--vd-amber, #e6b656) 30%, var(--vd-line, #2a2a30));
  }
  .row[data-status="missing"] {
    border-color: color-mix(in srgb, var(--vd-st-attention, #d27c7c) 30%, var(--vd-line, #2a2a30));
  }
  .row-head {
    display: flex;
    justify-content: space-between;
    align-items: baseline;
    gap: 8px;
  }
  .row-label {
    font-size: 13px;
    font-weight: 500;
  }
  .status {
    font-size: 11px;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    font-family: var(--vd-font-mono, monospace);
  }
  .layers {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 11px;
    font-family: var(--vd-font-mono, monospace);
    color: var(--vd-fg-3, #9a9aa2);
  }
  .layer {
    display: inline-flex;
    align-items: baseline;
    gap: 4px;
  }
  .layer-tag {
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }
  .layer-mark {
    font-weight: 700;
  }
  .row-actions {
    display: flex;
    gap: 6px;
    margin-top: 4px;
  }
  .verify-phrase {
    margin-top: 8px;
    padding: 10px;
    background: var(--vd-bg-0, #0e0e12);
    border: 0.5px solid var(--vd-line, #2a2a30);
    border-radius: 4px;
  }
  .phrase-label {
    margin: 0 0 6px;
    font-size: 11px;
    color: var(--vd-fg-3, #9a9aa2);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }
  .phrase {
    margin: 0 0 8px;
    padding: 8px;
    background: var(--vd-bg-2, #1d1d22);
    border-radius: 3px;
    font-size: 11.5px;
    line-height: 1.4;
    color: var(--vd-fg-2, #d0d0d6);
    white-space: pre-wrap;
    font-family: var(--vd-font-mono, monospace);
  }
  .phrase-hint {
    margin: 8px 0 0;
    font-size: 11px;
    color: var(--vd-fg-3, #9a9aa2);
    line-height: 1.45;
  }
  .show-all {
    align-self: flex-start;
    background: transparent;
    border: 0.5px solid var(--vd-line, #2a2a30);
    color: var(--vd-fg-3, #9a9aa2);
    padding: 4px 10px;
    border-radius: 999px;
    font-size: 11px;
    cursor: pointer;
    margin-top: 6px;
  }
  .show-all:hover {
    color: var(--vd-fg-1, #f0f0f0);
    border-color: var(--vd-fg-3, #9a9aa2);
  }
</style>
