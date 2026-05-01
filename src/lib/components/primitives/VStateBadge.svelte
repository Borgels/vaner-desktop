<!--
  VStateBadge — the 7px state dot used wherever the popover signals what
  Vaner is doing right now. A glow pads the dot so it reads against any bg.

  Mirrors `VStateBadge` from vaner-desktop-macos/vaner/Primitives/VStateBadge.swift.
  State enum maps to tokens.css `--vd-st-*` values.
-->
<script lang="ts">
  export type VState = "on" | "learning" | "prepared" | "active" | "attention" | "idle";
  type Props = { state: VState; size?: number };
  const { state, size = 7 }: Props = $props();
</script>

<span
  class="v-state-badge"
  data-state={state}
  style:--badge-size={`${size}px`}
></span>

<style>
  .v-state-badge {
    display: inline-block;
    width: var(--badge-size);
    height: var(--badge-size);
    border-radius: 50%;
    background: var(--badge-color, var(--vd-st-on));
    box-shadow: 0 0 calc(var(--badge-size) * 0.85)
      color-mix(in srgb, var(--badge-color, var(--vd-st-on)) 60%, transparent);
    flex: 0 0 auto;
  }
  .v-state-badge[data-state="on"]        { --badge-color: var(--vd-st-on); }
  .v-state-badge[data-state="learning"]  { --badge-color: var(--vd-st-learning); }
  .v-state-badge[data-state="prepared"]  { --badge-color: var(--vd-st-prepared); }
  .v-state-badge[data-state="active"]    { --badge-color: var(--vd-st-active); }
  .v-state-badge[data-state="attention"] { --badge-color: var(--vd-st-attention); }
  .v-state-badge[data-state="idle"]      {
    --badge-color: var(--vd-st-idle);
    opacity: 0.6;
    box-shadow: none;
  }
</style>
