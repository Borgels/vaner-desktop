<!--
  Spinner — the small amber arc used during install / setup-apply / refresh.
  Linear rotation; no easing changes (the macOS app explicitly does not use
  springs for the spinner, so neither do we).

  Mirrors `Spinner` from vaner-desktop-macos/vaner/Primitives/Spinner.swift.
-->
<script lang="ts">
  type Props = { size?: number };
  const { size = 12 }: Props = $props();
  const stroke = $derived(Math.max(1.4, size * 0.14));
</script>

<svg
  class="v-spinner"
  width={size}
  height={size}
  viewBox="0 0 24 24"
  aria-hidden="true"
>
  <circle
    cx="12"
    cy="12"
    r={12 - stroke}
    fill="none"
    stroke="var(--vd-line)"
    stroke-width={stroke}
  />
  <path
    d={`M 12 ${stroke} A ${12 - stroke} ${12 - stroke} 0 0 1 ${12 + (12 - stroke) * Math.sin((2 * Math.PI) * 0.28)} ${12 - (12 - stroke) * Math.cos((2 * Math.PI) * 0.28)}`}
    fill="none"
    stroke="var(--vd-amber)"
    stroke-width={stroke}
    stroke-linecap="round"
  />
</svg>

<style>
  .v-spinner {
    animation: v-spin 0.9s linear infinite;
  }
  @keyframes v-spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }
</style>
