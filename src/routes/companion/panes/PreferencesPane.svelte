<!--
  Preferences pane — silent hours toggle, persona tone sliders (chattiness,
  interrupt, learnDepth, voice), pause greyed out (daemon endpoint not
  shipped). Persisted to localStorage for v0.2.2 — WS8's daemon-side hooks
  pick them up later.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1Slider from "$lib/components/primitives/V1Slider.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import { silentHours } from "$lib/stores/silent-hours.js";

  // Persona tunables — load + persist to localStorage. Real engine wiring
  // arrives when the daemon ships POST /preferences/persona.
  type Persona = {
    chattiness: number;
    learnDepth: number;
    interrupt: "never" | "soft" | "firm";
    voice: "terse" | "precise" | "warm";
  };
  const defaults: Persona = {
    chattiness: 0.4,
    learnDepth: 0.5,
    interrupt: "soft",
    voice: "precise",
  };
  let persona = $state<Persona>(loadPersona());

  function loadPersona(): Persona {
    try {
      const raw = localStorage.getItem("vaner.pref.persona");
      if (!raw) return { ...defaults };
      return { ...defaults, ...JSON.parse(raw) };
    } catch {
      return { ...defaults };
    }
  }
  function savePersona() {
    try {
      localStorage.setItem("vaner.pref.persona", JSON.stringify(persona));
    } catch {
      /* noop */
    }
  }

  $effect(() => savePersona());
</script>

<header class="hd">
  <V1Kicker text="Preferences" />
  <V1Headline text="How Vaner sounds and when it speaks" size={22} />
  <V1Body
    muted
    text="Saved locally for now. Once the daemon ships its preferences endpoint these flow through to engine-side prompt-building and prediction throttling."
  />
</header>

<section class="block">
  <VSectionLabel text="Silent hours" />
  <label class="row toggle">
    <input type="checkbox" bind:checked={$silentHours} />
    <span class="row-text">
      <span class="row-title">Hold non-urgent moments overnight</span>
      <span class="row-detail">Vaner still indexes; it just doesn't surface anything until silent hours end.</span>
    </span>
  </label>
</section>

<section class="block">
  <VSectionLabel text="Pause" />
  <V1Body muted text="Coming with the daemon's POST /engine/pause endpoint (Tier B)." />
  <div class="actions">
    <V1GhostButton title="Pause Vaner" disabled />
  </div>
</section>

<section class="block">
  <VSectionLabel text="Tone" />
  <div class="sliders">
    <V1Slider bind:value={persona.chattiness} label={`Chattiness · ${(persona.chattiness * 100).toFixed(0)}%`} />
    <V1Slider bind:value={persona.learnDepth} label={`Learn depth · ${(persona.learnDepth * 100).toFixed(0)}%`} />
  </div>
  <div class="picker">
    <span class="picker-label">Interrupt</span>
    <select bind:value={persona.interrupt}>
      <option value="never">Never</option>
      <option value="soft">Soft</option>
      <option value="firm">Firm</option>
    </select>
    <span class="picker-label">Voice</span>
    <select bind:value={persona.voice}>
      <option value="terse">Terse</option>
      <option value="precise">Precise</option>
      <option value="warm">Warm</option>
    </select>
  </div>
</section>

<style>
  .hd { display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px; }
  .block { margin-bottom: 24px; }
  .row {
    display: flex;
    align-items: flex-start;
    gap: 10px;
    padding: 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    cursor: pointer;
  }
  .row.toggle input { margin-top: 3px; }
  .row-text { display: flex; flex-direction: column; gap: 2px; }
  .row-title { font-size: 13px; font-weight: 500; color: var(--vd-fg-1); }
  .row-detail { font-size: 11px; color: var(--vd-fg-3); }
  .sliders { margin-top: 10px; display: flex; flex-direction: column; gap: 14px; max-width: 420px; }
  .picker {
    margin-top: 16px;
    display: grid;
    grid-template-columns: max-content max-content;
    gap: 8px 12px;
    align-items: center;
  }
  .picker-label {
    font-size: 11px;
    color: var(--vd-fg-3);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  .picker select {
    background: var(--vd-bg-1);
    color: var(--vd-fg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
    padding: 6px 10px;
    font-family: var(--vd-font);
    font-size: 12px;
  }
  .actions { margin-top: 10px; }
</style>
