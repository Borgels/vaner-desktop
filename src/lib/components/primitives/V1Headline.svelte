<!--
  V1Headline — the popover's voice. Space Grotesk medium, slightly tightened
  letter-spacing, comfortable line-spacing. Each popover state uses one.

  Mirrors `V1Headline` from vaner-desktop-macos/vaner/Primitives/V1Typography.swift.
-->
<script lang="ts">
  type Props = {
    text: string;
    size?: number;
  };
  const { text, size = 15 }: Props = $props();
</script>

<h2 class="v1-headline" style:font-size={`${size}px`}>
  {text}
</h2>

<style>
  .v1-headline {
    margin: 0;
    font-family: var(--vd-font);
    font-weight: 500;
    letter-spacing: -0.011em;
    line-height: 1.36;
    color: var(--vd-fg-1);
  }
</style>
