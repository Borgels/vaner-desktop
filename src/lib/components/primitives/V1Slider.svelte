<!--
  V1Slider — gesture-driven slider for tone preferences (chattiness,
  interrupt, learnDepth, voice). Gradient fill purple→amber, 14px frame,
  the thumb is a 10px circle that scales on press.

  Mirrors `V1Slider` from vaner-desktop-macos/vaner/Primitives/V1Slider.swift.
-->
<script lang="ts">
  type Props = {
    value: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
    disabled?: boolean;
    onchange?: (v: number) => void;
  };
  let {
    value = $bindable(),
    min = 0,
    max = 1,
    step = 0.01,
    label,
    disabled = false,
    onchange,
  }: Props = $props();

  const pct = $derived(((value - min) / (max - min)) * 100);

  function handleInput(e: Event) {
    const v = parseFloat((e.target as HTMLInputElement).value);
    value = v;
    onchange?.(v);
  }
</script>

<div class="v-slider" class:disabled>
  {#if label}
    <span class="v-slider__label">{label}</span>
  {/if}
  <div class="v-slider__track">
    <div class="v-slider__fill" style:width={`${pct}%`}></div>
    <input
      type="range"
      {min}
      {max}
      {step}
      {value}
      {disabled}
      oninput={handleInput}
    />
  </div>
</div>

<style>
  .v-slider {
    display: flex;
    flex-direction: column;
    gap: 6px;
    width: 100%;
  }
  .v-slider__label {
    font-family: var(--vd-font);
    font-size: 12px;
    color: var(--vd-fg-2);
  }
  .v-slider__track {
    position: relative;
    height: 14px;
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.06);
    overflow: hidden;
  }
  .v-slider__fill {
    position: absolute;
    inset: 0 auto 0 0;
    background: linear-gradient(
      90deg,
      var(--vd-purple) 0%,
      var(--vd-amber) 100%
    );
    border-radius: 8px;
  }
  .v-slider input[type="range"] {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    margin: 0;
    background: transparent;
    appearance: none;
    -webkit-appearance: none;
    cursor: pointer;
  }
  .v-slider input[type="range"]::-webkit-slider-thumb {
    appearance: none;
    -webkit-appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--vd-fg-1);
    border: 1px solid color-mix(in srgb, var(--vd-purple) 60%, transparent);
    box-shadow: 0 0 0 2px rgba(0, 0, 0, 0.25);
    cursor: pointer;
    transition: transform 0.06s ease-out;
  }
  .v-slider input[type="range"]:active::-webkit-slider-thumb {
    transform: scale(1.1);
  }
  .v-slider input[type="range"]::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--vd-fg-1);
    border: 1px solid color-mix(in srgb, var(--vd-purple) 60%, transparent);
    cursor: pointer;
  }
  .v-slider.disabled {
    opacity: 0.4;
    pointer-events: none;
  }
</style>
