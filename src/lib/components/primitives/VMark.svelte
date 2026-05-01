<!--
  VMark — the brand dial. Purple ring + inner disc + amber tick + amber
  satellite. Brand colors are FIXED per the brand guide ("never recolor the
  mark outside the provided variants"); when a state-colored dot is needed
  it's overlaid ON TOP of the satellite at the same (17,6) position so the
  brand stays untouched but live state still reads.

  Mirrors `VMark` from vaner-desktop-macos/vaner/Primitives/VMark.swift.
  SVG paths come straight from
  vaner-desktop-macos/vaner/Assets.xcassets/BrandMark.imageset/mark-brand.svg
  so the popover, the macOS app, and the design canvas render identically.
-->
<script lang="ts">
  import type { VState } from "./VStateBadge.svelte";

  type Props = {
    size?: number;
    satelliteState?: VState | null;
    breathing?: boolean;
  };
  const { size = 28, satelliteState = null, breathing = false }: Props = $props();

  // Satellite center sits at viewBox (17, 6) with r=1.6 → diameter 3.2 in
  // a 22-unit viewBox. Pixel positions scale linearly with `size`.
  const satDiameter = $derived((size * 3.2) / 22);
  const satX = $derived(size * (17 / 22) - satDiameter / 2);
  const satY = $derived(size * (6 / 22) - satDiameter / 2);

  const stateVar: Record<VState, string> = {
    on: "var(--vd-st-on)",
    learning: "var(--vd-st-learning)",
    prepared: "var(--vd-st-prepared)",
    active: "var(--vd-st-active)",
    attention: "var(--vd-st-attention)",
    idle: "var(--vd-st-idle)",
  };

  const overlayColor = $derived(
    satelliteState && satelliteState !== "idle" ? stateVar[satelliteState] : null
  );
</script>

<div
  class="v-mark"
  style:--mark-size={`${size}px`}
  aria-hidden="true"
>
  <svg
    width={size}
    height={size}
    viewBox="0 0 22 22"
  >
    <circle cx="11" cy="11" r="9" fill="none" stroke="var(--vd-purple)" stroke-width="1.4" />
    <circle cx="11" cy="11" r="4" fill="var(--vd-purple)" />
    <path d="M11 11 L18 6" stroke="var(--vd-amber)" stroke-width="0.9" />
    <circle cx="18" cy="6" r="1.6" fill="var(--vd-amber)" />
  </svg>

  {#if overlayColor}
    <span
      class="v-mark__satellite"
      class:breathing
      style:--sat-color={overlayColor}
      style:--sat-size={`${satDiameter}px`}
      style:--sat-x={`${satX}px`}
      style:--sat-y={`${satY}px`}
    ></span>
  {/if}
</div>

<style>
  .v-mark {
    position: relative;
    display: inline-block;
    width: var(--mark-size);
    height: var(--mark-size);
    flex: 0 0 auto;
  }
  .v-mark svg {
    display: block;
  }
  .v-mark__satellite {
    position: absolute;
    left: var(--sat-x);
    top: var(--sat-y);
    width: var(--sat-size);
    height: var(--sat-size);
    border-radius: 50%;
    background: var(--sat-color);
    box-shadow: 0 0 calc(var(--sat-size) * 0.5)
      color-mix(in srgb, var(--sat-color) 60%, transparent);
    transform-origin: center;
  }
  .v-mark__satellite.breathing {
    animation: vd-breathe 3.2s ease-in-out infinite;
  }
</style>
