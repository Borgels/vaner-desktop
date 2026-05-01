<!--
  NavGlyph — conceptual icons for the companion-window sidebar. Maps to
  the same SF Symbols the macOS sibling uses in CompanionWindow.swift:

    prepared    → sparkle
    sources     → square.stack.3d.up
    agents      → person.2
    models      → brain
    engine      → bolt.circle
    preferences → slider.horizontal.3
    diagnostics → stethoscope

  Hand-rolled SVGs (rather than pulling lucide-svelte for ~7 icons).
  16×16 viewBox, currentColor, dim variant for unselected states.
-->
<script lang="ts">
  type Kind =
    | "prepared"
    | "sources"
    | "agents"
    | "models"
    | "engine"
    | "preferences"
    | "diagnostics";

  type Props = {
    kind: Kind | string;
    size?: number;
    dim?: boolean;
  };
  const { kind, size = 14, dim = false }: Props = $props();
  const k = $derived(String(kind).toLowerCase() as Kind);
</script>

<svg
  class="nav-glyph"
  class:dim
  width={size}
  height={size}
  viewBox="0 0 16 16"
  aria-hidden="true"
>
  {#if k === "prepared"}
    <!-- sparkle: 4-point star -->
    <path
      d="M8 2 L9 7 L14 8 L9 9 L8 14 L7 9 L2 8 L7 7 Z"
      fill="currentColor"
      opacity="0.92"
    />
  {:else if k === "sources"}
    <!-- stacked layers -->
    <path
      d="M2 6 L8 3 L14 6 L8 9 Z M3.5 8.5 L8 11 L12.5 8.5 M3.5 11 L8 13.5 L12.5 11"
      fill="none"
      stroke="currentColor"
      stroke-width="1.2"
      stroke-linejoin="round"
      stroke-linecap="round"
    />
  {:else if k === "agents"}
    <!-- two figures -->
    <circle cx="6" cy="5.5" r="2" fill="none" stroke="currentColor" stroke-width="1.2" />
    <circle cx="11" cy="6" r="1.6" fill="none" stroke="currentColor" stroke-width="1.1" />
    <path
      d="M2 13 C2 11 4 9.5 6 9.5 C8 9.5 10 11 10 13 M9 12 C9 10.5 10.2 9.5 11.5 9.5 C12.8 9.5 14 10.5 14 12"
      fill="none"
      stroke="currentColor"
      stroke-width="1.2"
      stroke-linecap="round"
    />
  {:else if k === "models"}
    <!-- brain — two lobes -->
    <path
      d="M4 6 C4 4 5 3 6.5 3 C7.4 3 8 3.5 8 4.5 C8 3.5 8.6 3 9.5 3 C11 3 12 4 12 6 C13 6.5 13 8 12 8.6 C12 10 11 11 9.5 11 C8.6 11 8 10.5 8 9.5 C8 10.5 7.4 11 6.5 11 C5 11 4 10 4 8.6 C3 8 3 6.5 4 6 Z"
      fill="none"
      stroke="currentColor"
      stroke-width="1.1"
      stroke-linejoin="round"
    />
    <path d="M8 4.5 L8 9.5" stroke="currentColor" stroke-width="0.9" opacity="0.6" />
  {:else if k === "engine"}
    <!-- lightning bolt in circle -->
    <circle cx="8" cy="8" r="5.5" fill="none" stroke="currentColor" stroke-width="1.1" />
    <path
      d="M9 4 L6 8.4 L8.2 8.4 L7 12 L10 7.6 L7.8 7.6 Z"
      fill="currentColor"
      opacity="0.9"
    />
  {:else if k === "preferences"}
    <!-- three horizontal sliders -->
    <path
      d="M2 4 L14 4 M2 8 L14 8 M2 12 L14 12"
      stroke="currentColor"
      stroke-width="1.1"
      stroke-linecap="round"
    />
    <circle cx="11" cy="4" r="1.6" fill="currentColor" />
    <circle cx="5" cy="8" r="1.6" fill="currentColor" />
    <circle cx="10" cy="12" r="1.6" fill="currentColor" />
  {:else if k === "diagnostics"}
    <!-- pulse line -->
    <path
      d="M2 8 L5 8 L6.5 5 L8 11 L9.5 8 L14 8"
      fill="none"
      stroke="currentColor"
      stroke-width="1.3"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  {:else}
    <circle cx="8" cy="8" r="5" fill="none" stroke="currentColor" stroke-width="1.2" />
  {/if}
</svg>

<style>
  .nav-glyph {
    color: var(--vd-fg-1);
    flex: 0 0 auto;
  }
  .nav-glyph.dim {
    color: var(--vd-fg-3);
  }
</style>
