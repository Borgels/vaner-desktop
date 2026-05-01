<!--
  QuietShell — the popover frame every state composes. Header (mark +
  wordmark + state label) → hairline → body slot → optional footer band.

  Mirrors `QuietShell` from vaner-desktop-macos/vaner/Primitives/QuietShell.swift.
  Used by every popover state component under
  src/lib/components/popover-states/.
-->
<script lang="ts">
  import VMark from "./VMark.svelte";
  import type { VState } from "./VStateBadge.svelte";

  type Props = {
    markState?: VState | null;
    breathingMark?: boolean;
    stateLabel: string;
    stateLabelTint?: string | null;
    children: import("svelte").Snippet;
    footer?: import("svelte").Snippet;
  };
  const {
    markState = null,
    breathingMark = false,
    stateLabel,
    stateLabelTint = null,
    children,
    footer,
  }: Props = $props();
</script>

<div class="quiet-shell">
  <!-- The header is the natural drag handle for the borderless popover.
       Decorationless windows on Linux can't be moved by the compositor
       without an explicit `data-tauri-drag-region`; making the whole
       header draggable matches what users expect. The brand + state
       label have no click handlers, so making them inert is fine. -->
  <header class="quiet-shell__head" data-tauri-drag-region>
    <span class="quiet-shell__brand" data-tauri-drag-region>
      <VMark size={22} satelliteState={markState} breathing={breathingMark} />
      <span class="quiet-shell__wordmark" data-tauri-drag-region>
        vaner<span class="quiet-shell__cursor" data-tauri-drag-region>_</span>
      </span>
    </span>
    <span
      class="quiet-shell__state"
      data-tauri-drag-region
      style:color={stateLabelTint ?? undefined}
    >
      {stateLabel}
    </span>
  </header>

  <div class="quiet-shell__hair"></div>

  <main class="quiet-shell__body">
    {@render children()}
  </main>

  {#if footer}
    <footer class="quiet-shell__foot">
      {@render footer()}
    </footer>
  {/if}
</div>

<style>
  .quiet-shell {
    display: flex;
    flex-direction: column;
    height: 100%;
    background: var(--vd-bg-0);
    color: var(--vd-fg-1);
  }
  .quiet-shell__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 13px 16px 11px;
    gap: 10px;
  }
  .quiet-shell__brand {
    display: inline-flex;
    align-items: center;
    gap: 9px;
  }
  .quiet-shell__wordmark {
    font-family: var(--vd-font-term);
    font-size: 14px;
    color: var(--vd-fg-1);
    letter-spacing: 0.02em;
  }
  .quiet-shell__cursor {
    color: var(--vd-amber);
    animation: vd-blink 1.6s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .quiet-shell__state {
    font-family: var(--vd-font);
    font-size: 10.5px;
    font-weight: 500;
    letter-spacing: 0.12em;
    text-transform: uppercase;
    color: var(--vd-fg-3);
  }
  .quiet-shell__hair {
    height: 0.5px;
    background: var(--vd-hair);
  }
  .quiet-shell__body {
    flex: 1 1 auto;
    overflow-y: auto;
    padding: 14px 16px 12px;
    scrollbar-width: thin;
    scrollbar-color: var(--vd-line) transparent;
  }
  .quiet-shell__body::-webkit-scrollbar {
    width: 6px;
  }
  .quiet-shell__body::-webkit-scrollbar-thumb {
    background: var(--vd-line);
    border-radius: 3px;
  }
  .quiet-shell__foot {
    background: rgba(0, 0, 0, 0.18);
    border-top: 0.5px solid var(--vd-hair);
    padding: 9px 14px;
    color: var(--vd-fg-4);
  }
</style>
