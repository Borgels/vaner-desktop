<!--
  VSectionLabel — companion-window section header. Same scale as V1Kicker
  but without uppercase letter-spacing and slightly less cramped, suitable
  for "Sources" / "Agents" / "Models" left-nav group titles.

  Mirrors `VSectionLabel` from vaner-desktop-macos/vaner/Primitives/.
-->
<script lang="ts">
  type Props = { text: string };
  const { text }: Props = $props();
</script>

<div class="v-section-label">{text}</div>

<style>
  .v-section-label {
    font-family: var(--vd-font);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.088em;
    text-transform: uppercase;
    color: var(--vd-fg-3);
    padding: 0 0 6px 0;
  }
</style>
