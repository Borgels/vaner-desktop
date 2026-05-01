<!--
  VMenuBarIcon — 18px wrapper around VMark, used inside the popover header.
  The actual Linux tray icon is a static PNG (src-tauri/icons/tray.png)
  rasterized from the brand mono SVG; this primitive is for the popover
  itself, not the tray.

  Mirrors `VMenuBarIcon` from vaner-desktop-macos/vaner/Primitives/.
-->
<script lang="ts">
  import VMark from "./VMark.svelte";
  import type { VState } from "./VStateBadge.svelte";

  type Props = {
    state?: VState | null;
    breathing?: boolean;
  };
  const { state = null, breathing = false }: Props = $props();
</script>

<VMark size={18} satelliteState={state} breathing={breathing} />
