<!--
  /setup — the in-app re-runnable wizard. Re-uses the same multi-slide
  SetupWizard component the dedicated /onboarding window mounts, so
  both surfaces stay visually consistent.

  Exit behavior depends on which Tauri window we're running inside:
    - "onboarding" window  →  invoke close_onboarding (close the window)
    - "main" popover        →  goto('/') (return to popover)
    - any other (companion) →  goto('/') (won't normally land here)
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import { invoke } from "@tauri-apps/api/core";
  import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";
  import SetupWizard from "$lib/components/SetupWizard.svelte";

  async function exit() {
    try {
      const label = getCurrentWebviewWindow().label;
      if (label === "onboarding") {
        await invoke("close_onboarding").catch(() => {});
        return;
      }
    } catch {
      /* fall through */
    }
    await goto("/");
  }
</script>

<SetupWizard onComplete={exit} onSkip={exit} />
