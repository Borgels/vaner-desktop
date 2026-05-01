<!--
  Models pane — surfaces the daemon's HardwareProfile (detected models,
  active picker) plus deep-run defaults. Read-only in v0.2.2; the active
  picker is wired to setup.apply when WS8 lands the endpoint.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import { setup, loadHardware } from "$lib/stores/setup.js";

  onMount(() => {
    loadHardware();
  });

  const hw = $derived($setup.hardware);
  // detected_models is a [runtime, model_id, size_label] tuple per
  // src/lib/contract/setup-types.ts:194.
  const models = $derived(hw?.detected_models ?? []);
</script>

<header class="hd">
  <V1Kicker text="Models" />
  <V1Headline text="Which models Vaner can use" size={22} />
  <V1Body
    muted
    text="Vaner picks a model that matches your hardware tier and posture. You can change it from the setup wizard; the active model below is pulled live from the daemon's hardware profile."
  />
</header>

{#if hw}
  <section class="block">
    <VSectionLabel text="Detected hardware" />
    <div class="kv">
      <span>Tier</span><span>{hw.tier}</span>
      <span>OS</span><span>{hw.os}</span>
      <span>CPU class</span><span>{hw.cpu_class}</span>
      <span>RAM</span><span>{hw.ram_gb} GB</span>
      <span>GPU</span><span>{hw.gpu}{hw.gpu_vram_gb ? ` · ${hw.gpu_vram_gb} GB VRAM` : ""}</span>
      <span>Runtimes</span><span>{hw.detected_runtimes.join(", ") || "—"}</span>
    </div>
  </section>

  {#if models.length > 0}
    <section class="block">
      <VSectionLabel text={`Detected models · ${models.length}`} />
      <ul class="models">
        {#each models as [runtime, modelId, sizeLabel] (modelId)}
          <li>
            <span class="m-name">{modelId}</span>
            <span class="m-meta">{runtime} · {sizeLabel}</span>
          </li>
        {/each}
      </ul>
    </section>
  {/if}
{:else}
  <section class="empty">
    <V1Body
      muted
      text="Hardware profile is loading… If this stays empty, the daemon may not be reachable on 127.0.0.1:8473."
    />
  </section>
{/if}

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
  .models {
    margin: 10px 0 0;
    padding: 0;
    list-style: none;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .models li {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
  }
  .m-name { font-size: 12.5px; color: var(--vd-fg-1); }
  .m-meta { font-family: var(--vd-font-mono); font-size: 11px; color: var(--vd-fg-3); }
  .empty { margin-top: 12px; }
</style>
