<!--
  V1PrimaryButton — solid pill. Default tint is `--vd-fg-1` (cream-on-black
  popover); pass an explicit `tint` (e.g. amber for adopt actions, attention
  red for engine-error retry) to override.

  Mirrors `V1PrimaryButton` from vaner-desktop-macos/vaner/Primitives/V1Buttons.swift.
-->
<script lang="ts">
  type Props = {
    title: string;
    tint?: string;
    disabled?: boolean;
    onclick?: (e: MouseEvent) => void;
  };
  const { title, tint, disabled = false, onclick }: Props = $props();
</script>

<button
  type="button"
  class="v1-primary"
  style:--btn-tint={tint ?? "var(--vd-fg-1)"}
  disabled={disabled}
  onclick={(e) => onclick?.(e)}
>
  {title}
</button>

<style>
  .v1-primary {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    padding: 7px 14px;
    font-family: var(--vd-font);
    font-size: 12px;
    font-weight: 500;
    color: var(--vd-bg-0);
    background: var(--btn-tint);
    border: 0;
    border-radius: var(--vd-r-chip);
    cursor: pointer;
    line-height: 1;
    letter-spacing: 0.005em;
    transition: filter 0.12s ease-out, transform 0.06s ease-out;
  }
  .v1-primary:hover:not(:disabled) {
    filter: brightness(0.95);
  }
  .v1-primary:active:not(:disabled) {
    transform: scale(0.98);
  }
  .v1-primary:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .v1-primary:focus-visible {
    outline: 2px solid color-mix(in srgb, var(--btn-tint) 80%, transparent);
    outline-offset: 2px;
  }
</style>
