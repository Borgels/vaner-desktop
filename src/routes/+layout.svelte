<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import { bootstrapUpdaterListeners } from "$lib/stores/updater.js";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import FirstRunGuidance from "$lib/components/FirstRunGuidance.svelte";
  import UpdateBanner from "$lib/components/UpdateBanner.svelte";

  let { children } = $props();

  onMount(() => {
    bootstrapAppStateListeners();
    bootstrapUpdaterListeners();
  });
</script>

<UpdateBanner />
{@render children?.()}
<ToastStack />
<FirstRunGuidance />
