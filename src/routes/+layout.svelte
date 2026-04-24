<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import FirstRunGuidance from "$lib/components/FirstRunGuidance.svelte";

  let { children } = $props();

  onMount(() => {
    bootstrapAppStateListeners();
  });
</script>

{@render children?.()}
<ToastStack />
<FirstRunGuidance />
