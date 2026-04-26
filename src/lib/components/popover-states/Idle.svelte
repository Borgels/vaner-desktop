<!--
  Idle — calm, off-duty. Different from Watching: this is the "engine running
  but nothing in flight" beat. Mirrors IdleView.swift mode=.idle.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
</script>

<QuietShell markState="idle" stateLabel="Off-duty">
  <V1Kicker text="On" />
  <div class="gap-6"></div>
  <V1Headline text="Nothing pressing." />
  <div class="gap-8"></div>
  <V1Body muted text="I'll speak up when something appears. Click the tray icon any time." />

  {#snippet footer()}
    <PopoverFooter health="idle" healthLabel="Ready when you are" />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .gap-8 { height: 8px; }
</style>
