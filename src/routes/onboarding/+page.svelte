<!--
  Onboarding window content. The multi-slide wizard lives in
  $lib/components/SetupWizard.svelte and is shared with /setup. Here we
  just hand it the right exit handlers — onComplete closes the window;
  onSkip also closes it (the layout will reopen onboarding on next launch
  while setup_status.completed_at is null, which is the intended UX).
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import SetupWizard from "$lib/components/SetupWizard.svelte";

  async function close() {
    await invoke("close_onboarding").catch(() => {});
  }
</script>

<svelte:head>
  <title>Welcome to Vaner</title>
</svelte:head>

<SetupWizard onComplete={close} onSkip={close} />
