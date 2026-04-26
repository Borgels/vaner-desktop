<!--
  Diagnostics pane — bundle version, daemon health probe, log tail, and
  a one-shot Send-incident button (writes a redacted bundle to disk).
  v0.2.2: surface the version + probe; the log tail and incident bundle
  flow once the daemon ships POST /diagnostics/incident.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import { getVersion } from "@tauri-apps/api/app";
  import { engineStatus } from "$lib/stores/engine-status.js";

  let appVersion = $state<string>("…");

  onMount(async () => {
    try {
      appVersion = await getVersion();
    } catch {
      appVersion = "unknown";
    }
  });
</script>

<header class="hd">
  <V1Kicker text="Diagnostics" />
  <V1Headline text="Help me help you" size={22} />
  <V1Body
    muted
    text="Snapshot of your local install. Use this view if something feels off — paste it into a bug report and I can usually triage in one round."
  />
</header>

<section class="block">
  <VSectionLabel text="App" />
  <div class="kv">
    <span>Bundle version</span><span>{appVersion}</span>
    <span>Built for</span><span>linux + windows · single repo</span>
  </div>
</section>

<section class="block">
  <VSectionLabel text="Engine" />
  <div class="kv">
    <span>Reachable</span><span>{$engineStatus.reachable ? "yes" : "no"}</span>
    <span>Files watched</span><span>{$engineStatus.filesWatched}</span>
    <span>Sources</span><span>{$engineStatus.sourcesCount}</span>
    <span>Uptime</span><span>{$engineStatus.uptimeMinutes}m</span>
  </div>
</section>

<section class="block">
  <VSectionLabel text="Actions" />
  <div class="actions">
    <V1PrimaryButton title="Send incident" />
    <V1GhostButton title="Open log dir" />
    <V1GhostButton title="Restart daemon" />
  </div>
</section>

<style>
  .hd { display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px; }
  .block { margin-bottom: 22px; }
  .kv {
    margin-top: 10px;
    display: grid;
    grid-template-columns: max-content 1fr;
    gap: 6px 16px;
    font-family: var(--vd-font);
    font-size: 12px;
  }
  .kv > span:nth-child(odd) { color: var(--vd-fg-3); text-transform: uppercase; letter-spacing: 0.05em; font-size: 10.5px; padding-top: 2px; }
  .kv > span:nth-child(even) { color: var(--vd-fg-1); font-family: var(--vd-font-mono); }
  .actions { display: flex; gap: 6px; margin-top: 10px; }
</style>
