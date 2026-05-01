<!--
  VContextCard — the prepared-moment card. Two visual densities controlled
  by `isLead`: lead cards get the `--vd-bg-2` brighter background and a
  15px title; supporting cards get `--vd-bg-1` and a 13px title.

  Mirrors `VContextCard` from vaner-desktop-macos/vaner/Primitives/VContextCard.swift.

  The `moment` shape is intentionally permissive — the canonical type lives
  in `src/lib/state/types.ts` (PreparedMoment) and includes title, prediction,
  source kind, source label, confidence (0..1), reasons (string[]). Optional
  fields are tolerated.
-->
<script lang="ts">
  import V1Kicker from "./V1Kicker.svelte";
  import SourceGlyph from "./SourceGlyph.svelte";

  type Moment = {
    title: string;
    prediction?: string;
    sourceKind?: string;
    sourceLabel?: string;
    confidence?: number;
    reasons?: string[];
    kicker?: string;
  };
  type Props = {
    moment: Moment;
    isLead?: boolean;
    onclick?: (e: MouseEvent) => void;
  };
  const { moment, isLead = false, onclick }: Props = $props();
  const conf = $derived(
    typeof moment.confidence === "number" ? Math.round(moment.confidence * 100) : null
  );
</script>

<button
  type="button"
  class="v-card"
  class:lead={isLead}
  onclick={(e) => onclick?.(e)}
>
  <div class="v-card__head">
    {#if moment.kicker}
      <V1Kicker text={moment.kicker} />
    {/if}
    {#if moment.sourceKind || moment.sourceLabel}
      <span class="v-card__source">
        {#if moment.sourceKind}
          <SourceGlyph kind={moment.sourceKind} size={12} dim />
        {/if}
        {#if moment.sourceLabel}
          <span>{moment.sourceLabel}</span>
        {/if}
      </span>
    {/if}
  </div>

  <div class="v-card__title" class:large={isLead}>{moment.title}</div>

  {#if moment.prediction}
    <div class="v-card__prediction" class:large={isLead}>{moment.prediction}</div>
  {/if}

  {#if isLead && moment.reasons && moment.reasons.length > 0}
    <ul class="v-card__reasons">
      {#each moment.reasons as r (r)}
        <li>
          <span class="v-card__bullet"></span>
          <span>{r}</span>
        </li>
      {/each}
    </ul>
  {/if}

  {#if conf !== null}
    <div class="v-card__foot">
      <span class="v-card__conf">{conf}%</span>
    </div>
  {/if}
</button>

<style>
  .v-card {
    display: block;
    width: 100%;
    text-align: left;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    padding: 11px 13px;
    cursor: pointer;
    color: inherit;
    font: inherit;
    transition: background 0.12s ease-out, border-color 0.12s ease-out;
  }
  .v-card.lead {
    background: var(--vd-bg-2);
    padding: 14px 15px 13px;
  }
  .v-card:hover {
    background: var(--vd-bg-2);
    border-color: rgba(255, 255, 255, 0.14);
  }
  .v-card.lead:hover {
    background: color-mix(in srgb, var(--vd-bg-2) 90%, white 10%);
  }

  .v-card__head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    margin-bottom: 6px;
  }
  .v-card__source {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-family: var(--vd-font);
    font-size: var(--vd-t-mini);
    color: var(--vd-fg-3);
  }
  .v-card__title {
    font-family: var(--vd-font);
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    line-height: 1.3;
  }
  .v-card__title.large {
    font-size: 15px;
    line-height: 1.32;
  }
  .v-card__prediction {
    margin-top: 6px;
    font-family: var(--vd-font);
    font-size: 12px;
    color: var(--vd-fg-2);
    line-height: 1.5;
  }
  .v-card__prediction.large {
    font-size: 12.5px;
    color: var(--vd-fg-2);
  }
  .v-card__reasons {
    list-style: none;
    margin: 10px 0 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 5px;
  }
  .v-card__reasons li {
    display: flex;
    gap: 8px;
    font-family: var(--vd-font);
    font-size: 12px;
    color: var(--vd-fg-2);
    line-height: 1.42;
  }
  .v-card__bullet {
    flex: 0 0 auto;
    width: 5px;
    height: 5px;
    margin-top: 7px;
    border-radius: 50%;
    background: var(--vd-amber);
    opacity: 0.9;
  }
  .v-card__foot {
    margin-top: 10px;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .v-card__conf {
    font-family: var(--vd-font-mono);
    font-size: 11px;
    color: var(--vd-fg-3);
    font-variant-numeric: tabular-nums;
  }

  .v-card:focus-visible {
    outline: 2px solid var(--vd-fg-3);
    outline-offset: 2px;
  }
</style>
