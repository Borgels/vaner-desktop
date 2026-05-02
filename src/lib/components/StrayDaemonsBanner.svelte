<!--
  StrayDaemonsBanner — at-the-top-of-popover prompt that fires when
  the Rust startup audit found `vaner daemon / up / proxy / mcp`
  processes the desktop didn't spawn. The desktop owns daemon
  lifecycle; uninvited daemons compete for resources (often the model
  loop), so the user gets one click to reconcile.

  Sibling to UpdateBanner — same visual slot, both render
  conditionally above the active popover state.
-->
<script lang="ts">
  import {
    daemonStrays,
    dismissDaemonStrays,
    killStrays,
  } from "$lib/stores/daemon-audit.js";

  async function stopAll() {
    const pids = $daemonStrays.map((s) => s.pid);
    await killStrays(pids);
  }
</script>

{#if $daemonStrays.length > 0}
  <div class="banner" role="status" aria-live="polite">
    <div class="copy">
      <span class="label">Stray daemons</span>
      <span class="msg">
        {$daemonStrays.length}
        {$daemonStrays.length === 1 ? "process is" : "processes are"} running
        outside the desktop. Vaner Desktop owns the daemon — these are stale.
      </span>
      <details class="trace">
        <summary>{$daemonStrays.length === 1 ? "Show it" : "Show them"}</summary>
        <ul>
          {#each $daemonStrays as p (p.pid)}
            <li>
              <code>PID {p.pid}</code>
              <span class="kind">{p.kind}</span>
              {#if p.path}<span class="path">{p.path}</span>{/if}
              <code class="cmd">{p.cmdline}</code>
            </li>
          {/each}
        </ul>
      </details>
    </div>
    <div class="actions">
      <button class="stop" onclick={stopAll}>Stop them</button>
      <button class="later" onclick={dismissDaemonStrays}>Later</button>
    </div>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 14px;
    background: color-mix(in srgb, var(--vd-st-attention) 7%, var(--vd-bg-1));
    border-bottom: 0.5px solid color-mix(in srgb, var(--vd-st-attention) 30%, transparent);
  }
  .copy {
    display: flex;
    flex-direction: column;
    gap: 4px;
    min-width: 0;
    flex: 1 1 auto;
  }
  .label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1px;
    text-transform: uppercase;
    color: var(--vd-st-attention);
  }
  .msg {
    font-size: 11.5px;
    color: var(--vd-fg-2);
    line-height: 1.4;
  }
  .trace {
    font-size: 10.5px;
    color: var(--vd-fg-3);
  }
  .trace summary {
    cursor: pointer;
    user-select: none;
  }
  .trace ul {
    margin: 6px 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 4px;
    max-height: 140px;
    overflow: auto;
  }
  .trace li {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    align-items: baseline;
  }
  .trace code {
    font-family: var(--vd-font-mono);
    color: var(--vd-fg-2);
  }
  .trace .kind {
    font-weight: 600;
    color: var(--vd-amber);
  }
  .trace .path {
    color: var(--vd-fg-3);
  }
  .trace .cmd {
    flex: 1 1 100%;
    color: var(--vd-fg-3);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .actions {
    display: flex;
    gap: 6px;
    align-items: flex-start;
    flex: 0 0 auto;
  }
  button {
    font-family: var(--vd-font);
    font-size: 11px;
    padding: 5px 10px;
    border-radius: 5px;
    border: 0.5px solid var(--vd-line);
    background: transparent;
    color: var(--vd-fg-2);
    cursor: pointer;
  }
  button.stop {
    background: var(--vd-st-attention);
    color: var(--vd-bg-0);
    border-color: transparent;
    font-weight: 500;
  }
</style>
