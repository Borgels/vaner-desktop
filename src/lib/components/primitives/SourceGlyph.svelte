<!--
  SourceGlyph — small glyph identifying which source a prepared moment
  came from. macOS uses SF Symbols; on Linux we hand-roll the tiny set of
  shapes the popover actually surfaces, so we don't pull in lucide-svelte
  for ten icons we'd subset anyway.

  Mirrors `SourceGlyph` from vaner-desktop-macos/vaner/Primitives/SourceGlyph.swift.

  Recognized kinds (case-insensitive):
    github · gitlab · files · linear · slack · calendar · drive ·
    mail · notion · web · code · agent · unknown
-->
<script lang="ts">
  type Kind =
    | "github"
    | "gitlab"
    | "files"
    | "linear"
    | "slack"
    | "calendar"
    | "drive"
    | "mail"
    | "notion"
    | "web"
    | "code"
    | "agent"
    | "unknown";

  type Props = {
    kind: Kind | string;
    size?: number;
    dim?: boolean;
  };
  const { kind, size = 14, dim = false }: Props = $props();
  const k = $derived(String(kind).toLowerCase() as Kind);
</script>

<svg
  class="source-glyph"
  class:dim
  width={size}
  height={size}
  viewBox="0 0 16 16"
  aria-hidden="true"
>
  {#if k === "github"}
    <!-- chevron-left forwardslash chevron-right -->
    <path d="M5 4 L2 8 L5 12 M11 4 L14 8 L11 12 M9 3 L7 13" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" stroke-linejoin="round" />
  {:else if k === "gitlab"}
    <path d="M8 14 L2 8 L4 3 L6 7 L10 7 L12 3 L14 8 Z" fill="currentColor" opacity="0.85" />
  {:else if k === "files"}
    <path d="M2 5 L2 12 A1 1 0 0 0 3 13 L13 13 A1 1 0 0 0 14 12 L14 6 A1 1 0 0 0 13 5 L8 5 L7 4 L3 4 A1 1 0 0 0 2 5 Z" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
  {:else if k === "linear"}
    <path d="M2 8 L8 14 M2 5 L11 14 M2 2 L14 14 M5 2 L14 11 M8 2 L14 8" fill="none" stroke="currentColor" stroke-width="1.1" stroke-linecap="round" />
  {:else if k === "slack"}
    <rect x="3" y="6" width="3" height="3" rx="1" fill="currentColor" opacity="0.85" />
    <rect x="7" y="2" width="3" height="3" rx="1" fill="currentColor" opacity="0.85" />
    <rect x="10" y="7" width="3" height="3" rx="1" fill="currentColor" opacity="0.85" />
    <rect x="6" y="11" width="3" height="3" rx="1" fill="currentColor" opacity="0.85" />
  {:else if k === "calendar"}
    <rect x="2" y="4" width="12" height="10" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M2 7 L14 7 M5 2 L5 5 M11 2 L11 5" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
  {:else if k === "drive"}
    <path d="M2 12 L5 6 L11 6 L14 12 Z M5 6 L8 12 M11 6 L8 12" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
  {:else if k === "mail"}
    <rect x="2" y="4" width="12" height="9" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M2 5 L8 9 L14 5" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linejoin="round" />
  {:else if k === "notion"}
    <rect x="3" y="2" width="10" height="12" rx="1.5" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M5 5 L11 11 M5 5 L5 11 M11 5 L11 11" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
  {:else if k === "web"}
    <circle cx="8" cy="8" r="6" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M2 8 L14 8 M8 2 C5 5 5 11 8 14 C11 11 11 5 8 2" fill="none" stroke="currentColor" stroke-width="1.2" />
  {:else if k === "code"}
    <path d="M5 5 L2 8 L5 11 M11 5 L14 8 L11 11 M10 4 L6 12" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
  {:else if k === "agent"}
    <circle cx="8" cy="6" r="3" fill="none" stroke="currentColor" stroke-width="1.2" />
    <path d="M3 14 C3 11 5 9 8 9 C11 9 13 11 13 14" fill="none" stroke="currentColor" stroke-width="1.2" stroke-linecap="round" />
  {:else}
    <circle cx="8" cy="8" r="5" fill="none" stroke="currentColor" stroke-width="1.2" />
    <text x="8" y="11" text-anchor="middle" font-family="var(--vd-font-mono)" font-size="7" fill="currentColor">?</text>
  {/if}
</svg>

<style>
  .source-glyph {
    color: var(--vd-fg-1);
    flex: 0 0 auto;
  }
  .source-glyph.dim {
    color: var(--vd-fg-4);
  }
</style>
