<script lang="ts">
  import "../app.css";
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import { page } from "$app/stores";
  import { bootstrapAppStateListeners } from "$lib/stores/app-state.js";
  import { bootstrapUpdaterListeners } from "$lib/stores/updater.js";
  import { loadStatus } from "$lib/stores/setup.js";
  import ToastStack from "$lib/components/ToastStack.svelte";
  import FirstRunGuidance from "$lib/components/FirstRunGuidance.svelte";
  import UpdateBanner from "$lib/components/UpdateBanner.svelte";

  let { children } = $props();

  // Tracks whether the setup wizard has run. The FirstRunGuidance
  // (GNOME app-indicator nudge) should only fire after setup
  // completes; until then it stays suppressed.
  let setupCompleted: boolean = $state(false);

  onMount(async () => {
    bootstrapAppStateListeners();
    bootstrapUpdaterListeners();

    // First-run check: if there's no [setup] section yet, route to
    // the wizard. The wizard is dismissible; the next launch fires
    // the check again until the user applies an answer set.
    try {
      const status = await loadStatus();
      const completedAt = status?.setup?.completed_at;
      setupCompleted = !!completedAt;
      if (!setupCompleted) {
        // Don't redirect if the user is already on the setup page
        // (or any nested setup route).
        const currentPath = $page.url.pathname;
        if (!currentPath.startsWith("/setup")) {
          await goto("/setup");
        }
      }
    } catch {
      // Daemon / CLI unreachable: leave the popover on its default
      // route; the user can run setup later from Preferences.
      setupCompleted = false;
    }
  });
</script>

<UpdateBanner />
{@render children?.()}
<ToastStack />
{#if setupCompleted}
  <FirstRunGuidance />
{/if}
