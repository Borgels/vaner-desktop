<!--
  NotInstalled — engine binary is not on PATH and not in any known location.
  Linux equivalent of the macOS "engine missing" screen, simplified for the
  Tauri context. Mirrors IdleView.swift mode=.notInstalled.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
</script>

<QuietShell markState="idle" stateLabel="Engine missing" stateLabelTint="var(--vd-st-attention)">
  <V1Kicker text="Engine missing" color="var(--vd-st-attention)" />
  <div class="gap-6"></div>
  <V1Headline text="Vaner needs to install its engine." />
  <div class="gap-8"></div>
  <V1Body
    muted
    text="The CLI (`vaner`) wasn't found on $PATH and isn't in $VANER_BIN. Install via pipx or pip; it talks to this app over loopback."
  />
  <pre class="cmd">pipx install "vaner[mcp]"</pre>

  <div class="actions">
    <V1PrimaryButton title="Open install guide" />
    <V1GhostButton title="Set $VANER_BIN…" />
  </div>

  {#snippet footer()}
    <PopoverFooter health="idle" healthLabel="No engine found" detailsDisabled />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .gap-8 { height: 8px; }
  .cmd {
    margin: 14px 0 0;
    padding: 8px 10px;
    background: rgba(255, 255, 255, 0.04);
    border: 0.5px solid var(--vd-hair);
    border-radius: 6px;
    font-family: var(--vd-font-mono);
    font-size: 11.5px;
    color: var(--vd-fg-1);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
