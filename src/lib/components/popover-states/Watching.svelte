<!--
  Watching — connected, on, alive, nothing strong yet. Calm idle voice.
  Mirrors WatchingView.swift + handoff V1Watching.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import type { WatchingSummary } from "$lib/state/types.js";

  type Props = { summary: WatchingSummary; silentHours: boolean };
  const { summary, silentHours }: Props = $props();
</script>

<QuietShell markState="on" stateLabel={silentHours ? "Silent hours" : "Watching"}>
  {#if silentHours}
    <div class="silent">
      <V1Kicker text="Silent hours" color="var(--vd-purple)" />
      <div class="gap-6"></div>
      <V1Body muted text="Holding new prepared moments and surfacing them when silent hours end." />
    </div>
  {:else}
    <V1Kicker text="On" />
    <div class="gap-6"></div>
    <V1Headline text="Nothing strong enough yet." />
    <div class="gap-8"></div>
    <V1Body
      muted
      text={`Watching ${summary.filesWatched} files across ${summary.sourcesCount} sources. I'll surface a moment as soon as one is worth your attention.`}
    />
    <div class="actions">
      <V1GhostButton title="Lower the bar" />
      <V1GhostButton title="Pause Vaner" disabled />
    </div>
  {/if}

  {#snippet footer()}
    <PopoverFooter health="on" healthLabel={`${summary.filesWatched} files watched`} />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .gap-8 { height: 8px; }
  .silent {
    padding: 8px 12px;
    background: color-mix(in srgb, var(--vd-purple) 12%, transparent);
    border-radius: 8px;
    border: 0.5px solid color-mix(in srgb, var(--vd-purple) 30%, transparent);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
