<!--
  VMenuRow — companion-window left-nav row primitive. Optional leading
  icon + title + optional trailing detail. Click to fire the bound action.

  Mirrors `VMenuRow` from vaner-desktop-macos/vaner/Primitives/VMenuRow.swift.
-->
<script lang="ts">
  type Props = {
    title: string;
    detail?: string;
    selected?: boolean;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
    icon?: import("svelte").Snippet;
  };
  const {
    title,
    detail,
    selected = false,
    disabled = false,
    onclick,
    icon,
  }: Props = $props();
</script>

<button
  type="button"
  class="v-menu-row"
  class:selected
  disabled={disabled}
  onclick={(e) => onclick?.(e)}
>
  {#if icon}
    <span class="v-menu-row__icon">
      {@render icon()}
    </span>
  {/if}
  <span class="v-menu-row__title">{title}</span>
  {#if detail}
    <span class="v-menu-row__detail">{detail}</span>
  {/if}
</button>

<style>
  .v-menu-row {
    display: flex;
    align-items: center;
    gap: 9px;
    width: 100%;
    padding: 8px 10px;
    background: transparent;
    color: var(--vd-fg-1);
    border: 0;
    border-radius: var(--vd-r-chip);
    font-family: var(--vd-font);
    font-size: 13px;
    text-align: left;
    cursor: pointer;
    transition: background 0.12s ease-out;
  }
  .v-menu-row:hover:not(:disabled) {
    background: var(--vd-bg-2);
  }
  .v-menu-row.selected {
    background: color-mix(in srgb, var(--vd-purple) 18%, transparent);
  }
  .v-menu-row:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .v-menu-row__icon {
    flex: 0 0 auto;
    width: 16px;
    height: 16px;
    color: var(--vd-fg-2);
    display: inline-flex;
    align-items: center;
    justify-content: center;
  }
  .v-menu-row__title {
    flex: 1 1 auto;
    color: var(--vd-fg-1);
  }
  .v-menu-row__detail {
    flex: 0 0 auto;
    font-family: var(--vd-font-mono);
    font-size: 11px;
    color: var(--vd-fg-3);
    font-variant-numeric: tabular-nums;
  }
</style>
