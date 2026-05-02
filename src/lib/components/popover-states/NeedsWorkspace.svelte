<!--
  NeedsWorkspace — first-launch picker. The desktop drives the Vaner
  CLI against a single repo path; before the user picks one we have
  nowhere to point `vaner up` / `vaner status`, so this state takes
  precedence over notInstalled / engineMissing / error.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import { pickWorkspace } from "$lib/stores/workspace.js";
  import { showToast } from "$lib/stores/toast.js";

  let picking = $state(false);

  async function pick() {
    if (picking) return;
    picking = true;
    try {
      const path = await pickWorkspace();
      if (path) {
        showToast(`Workspace set to ${path}`, "success", 3000);
      }
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not set workspace: ${err}`,
        "attention",
        4000,
      );
    } finally {
      picking = false;
    }
  }
</script>

<QuietShell markState="idle" stateLabel="Pick a workspace" stateLabelTint="var(--vd-amber)">
  <V1Headline text="Which folder should Vaner watch?" />
  <div class="gap-8"></div>
  <V1Body
    muted
    text="Vaner indexes one repository at a time. Pick the project folder you want it to watch — usually the root of a git checkout. You can change this later in Preferences."
  />

  <div class="actions">
    <V1PrimaryButton
      title={picking ? "Opening…" : "Pick a folder"}
      tint="var(--vd-amber)"
      onclick={pick}
    />
  </div>

  {#snippet footer()}
    <PopoverFooter health="idle" detailsDisabled />
  {/snippet}
</QuietShell>

<style>
  .gap-8 { height: 8px; }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
