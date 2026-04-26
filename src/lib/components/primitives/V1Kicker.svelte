<!--
  V1Kicker — small uppercase label that sits above headlines. The Vaner
  popover uses a kicker on every state ("PREPARED", "LEARNING", "NEEDS YOU"),
  treating it as the state badge's textual sibling.

  Mirrors `V1Kicker` from vaner-desktop-macos/vaner/Primitives/V1Typography.swift.
-->
<script lang="ts">
  type Props = {
    text: string;
    color?: string;
  };
  const { text, color }: Props = $props();
</script>

<span class="v1-kicker" style:color={color}>
  {text}
</span>

<style>
  .v1-kicker {
    display: inline-block;
    font-family: var(--vd-font);
    font-size: var(--vd-t-mini);
    font-weight: 500;
    letter-spacing: 0.14em;
    text-transform: uppercase;
    color: var(--vd-fg-3);
    line-height: 1;
  }
</style>
