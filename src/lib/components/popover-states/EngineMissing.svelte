<!--
  EngineMissing — install flow. Renders one of three sub-states based on
  the InstallFlowState payload. Mirrors EngineMissingView.swift +
  InstallingView.swift.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { InstallFlowState } from "$lib/state/types.js";

  type Props = { install: InstallFlowState };
  const { install }: Props = $props();
</script>

<QuietShell markState="idle" stateLabel="Engine missing">
  {#if install.kind === "checkingForEngine"}
    <div class="row-mid">
      <Spinner size={16} />
      <V1Body muted text="Looking for the engine binary…" />
    </div>
  {:else if install.kind === "installing"}
    <V1Kicker text="Installing engine" />
    <div class="gap-6"></div>
    <V1Headline text={`Installing… ${Math.round(install.progress * 100)}%`} />
    <div class="bar"><div class="bar-fill" style:width={`${install.progress * 100}%`}></div></div>
    {#if install.logTail.length}
      <pre class="log">{install.logTail.slice(-6).join("\n")}</pre>
    {/if}
  {:else if install.kind === "installed"}
    <V1Kicker text="Engine ready" color="var(--vd-st-on)" />
    <div class="gap-6"></div>
    <V1Headline text={`vaner ${install.version} installed.`} />
    <V1Body muted text="The popover will switch to the live state in a moment." />
  {:else if install.kind === "upgradeAvailable"}
    <V1Kicker text="Upgrade available" />
    <div class="gap-6"></div>
    <V1Headline text={`vaner ${install.latest} is out (you have ${install.current}).`} />
    <div class="actions">
      <V1PrimaryButton title="Upgrade now" />
      <V1GhostButton title="Later" />
    </div>
  {:else if install.kind === "failed"}
    <V1Kicker text="Install failed" color="var(--vd-st-attention)" />
    <div class="gap-6"></div>
    <V1Headline text="Couldn't install the engine." />
    <V1Body muted text={install.message} />
    <div class="actions">
      <V1PrimaryButton title="Try again" />
      <V1GhostButton title="Open install guide" />
    </div>
  {:else}
    <V1Kicker text="Engine not detected" />
    <div class="gap-6"></div>
    <V1Headline text="Vaner needs to install its engine." />
    <V1Body muted text="One-time setup. After this, the engine runs on loopback in the background." />
    <div class="actions">
      <V1PrimaryButton title="Install engine" />
      <V1GhostButton title="Show me how, manually" />
    </div>
  {/if}

  {#snippet footer()}
    <PopoverFooter health="idle" healthLabel="Engine missing" detailsDisabled />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .row-mid {
    display: flex;
    gap: 10px;
    align-items: center;
    padding: 10px 0;
  }
  .bar {
    margin-top: 10px;
    height: 6px;
    background: rgba(255, 255, 255, 0.06);
    border-radius: 3px;
    overflow: hidden;
  }
  .bar-fill {
    height: 100%;
    background: linear-gradient(90deg, var(--vd-purple), var(--vd-amber));
    transition: width 0.3s ease-out;
  }
  .log {
    margin: 12px 0 0;
    padding: 8px 10px;
    background: rgba(0, 0, 0, 0.25);
    border: 0.5px solid var(--vd-hair);
    border-radius: 6px;
    font-family: var(--vd-font-mono);
    font-size: 10.5px;
    color: var(--vd-fg-3);
    max-height: 100px;
    overflow: auto;
    white-space: pre;
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
