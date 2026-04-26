<!--
  Engine pane — wraps the existing /preferences/EnginePanel.svelte, which
  already shows current setup answers, active bundle, refresh-daemon
  button. Reskin done at the container level; the panel itself uses the
  new tokens via cascade.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import EnginePanel from "../../preferences/EnginePanel.svelte";
</script>

<header class="hd">
  <V1Kicker text="Engine" />
  <V1Headline text="How Vaner is configured today" size={22} />
  <V1Body
    muted
    text="Read-only summary of your current bundle, work-style answers, and hardware tier. Re-run the wizard from here to change them."
  />
</header>

<section class="panel">
  <EnginePanel />
</section>

<style>
  .hd { display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px; }
  .panel {
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    background: var(--vd-bg-1);
    padding: 16px;
  }
</style>
