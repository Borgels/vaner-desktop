<!--
  V1Body — paragraph text. 12px (one notch below the 13px UI body font on
  macOS, where the popover assumes a Retina HiDPI render). On Linux Tauri
  WebKitGTK the same 12px reads at the same angular size.

  Mirrors `V1Body` from vaner-desktop-macos/vaner/Primitives/V1Typography.swift.
-->
<script lang="ts">
  type Props = {
    text?: string;
    muted?: boolean;
    children?: import("svelte").Snippet;
  };
  const { text, muted = false, children }: Props = $props();
</script>

<p class="v1-body" class:muted>
  {#if children}
    {@render children()}
  {:else}
    {text}
  {/if}
</p>

<style>
  .v1-body {
    margin: 0;
    font-family: var(--vd-font);
    font-size: 12px;
    font-weight: 400;
    line-height: 1.5;
    color: var(--vd-fg-1);
  }
  .v1-body.muted {
    color: var(--vd-fg-2);
  }
</style>
