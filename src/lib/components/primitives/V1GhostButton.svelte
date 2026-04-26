<!--
  V1GhostButton — secondary action. Hairline stroke, no fill, picks up
  attention red when `destructive`.

  Mirrors `V1GhostButton` from vaner-desktop-macos/vaner/Primitives/V1Buttons.swift.
-->
<script lang="ts">
  type Props = {
    title: string;
    destructive?: boolean;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
  };
  const { title, destructive = false, disabled = false, onclick }: Props = $props();
</script>

<button
  type="button"
  class="v1-ghost"
  class:destructive
  disabled={disabled}
  onclick={(e) => onclick?.(e)}
>
  {title}
</button>

<style>
  .v1-ghost {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 6px 13px;
    font-family: var(--vd-font);
    font-size: 12px;
    font-weight: 400;
    color: var(--vd-fg-2);
    background: transparent;
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
    cursor: pointer;
    line-height: 1;
    transition: background 0.12s ease-out, border-color 0.12s ease-out, color 0.12s ease-out;
  }
  .v1-ghost:hover:not(:disabled) {
    background: var(--vd-bg-2);
    color: var(--vd-fg-1);
  }
  .v1-ghost:active:not(:disabled) {
    background: var(--vd-bg-1);
  }
  .v1-ghost:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .v1-ghost.destructive {
    color: var(--vd-st-attention);
    border-color: color-mix(in srgb, var(--vd-st-attention) 35%, transparent);
  }
  .v1-ghost.destructive:hover:not(:disabled) {
    background: color-mix(in srgb, var(--vd-st-attention) 12%, transparent);
    color: var(--vd-st-attention);
  }
  .v1-ghost:focus-visible {
    outline: 2px solid var(--vd-fg-3);
    outline-offset: 2px;
  }
</style>
