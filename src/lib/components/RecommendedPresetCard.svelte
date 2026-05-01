<!--
  Recommended setup card.

  v0.2.3 simplification (per user feedback):
    - Single flat card; the wizard's outer slide no longer wraps it in
      another card-like header.
    - Drops the "Setup will…" install-plan list and the runner/model
      availability chips. Hardware + chosen model is enough.
    - Adds a Light / Medium / Heavy segmented switch backed by the
      registry's alternatives list, so power users can override the
      automatic pick. Active size + model id renders as a single muted
      caption underneath.

  The override id is reported via the `selectedModelId` bindable prop;
  the wizard reads it on apply and runs `vaner config set backend.model`
  after the policy bundle is written.
-->
<script lang="ts">
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import type {
    ModelsRecommendedPayload,
    RecommendedModelEntry,
  } from "$lib/stores/setup.js";
  import type { HardwareProfile } from "$lib/contract/setup-types.js";

  type SizeKey = "light" | "medium" | "heavy";

  type Props = {
    payload: ModelsRecommendedPayload | null;
    loading: boolean;
    hardware?: HardwareProfile | null;
    /** Bound: the model id the user has chosen (may equal the auto pick). */
    selectedModelId?: string | null;
  };
  let {
    payload,
    loading,
    hardware = null,
    selectedModelId = $bindable(null),
  }: Props = $props();

  const autoPick = $derived<RecommendedModelEntry | null>(
    payload?.user?.selected_model ?? payload?.selected ?? null,
  );
  const runtime = $derived(payload?.user?.selected_runtime ?? null);

  // Bucket every available model (auto pick + alternatives) into a
  // size lane. The lane thresholds are intentionally crude — the goal is
  // a one-glance choice, not a fine-grained slider.
  const allCandidates = $derived<RecommendedModelEntry[]>(
    [autoPick, ...(payload?.alternatives ?? [])]
      .filter((m): m is RecommendedModelEntry => Boolean(m))
      .filter(
        (m, idx, arr) =>
          arr.findIndex((other) => modelKey(other) === modelKey(m)) === idx,
      ),
  );

  function modelKey(m: RecommendedModelEntry): string {
    return m.model_id ?? m.id ?? m.family ?? "";
  }

  function modelMemoryGb(m: RecommendedModelEntry): number {
    return (
      m.recommended_effective_memory_gb ??
      m.min_effective_memory_gb ??
      m.min_effective_gb_q4 ??
      0
    );
  }

  function bucketFor(m: RecommendedModelEntry): SizeKey {
    const gb = modelMemoryGb(m);
    if (gb <= 12) return "light";
    if (gb <= 24) return "medium";
    return "heavy";
  }

  // Pick one model per bucket — prefer the registry's auto pick when it
  // belongs to the bucket, otherwise the candidate with the lowest memory
  // footprint inside that bucket (most installable).
  const byBucket = $derived<Record<SizeKey, RecommendedModelEntry | null>>({
    light: pickForBucket("light"),
    medium: pickForBucket("medium"),
    heavy: pickForBucket("heavy"),
  });

  function pickForBucket(bucket: SizeKey): RecommendedModelEntry | null {
    const inBucket = allCandidates.filter((m) => bucketFor(m) === bucket);
    if (inBucket.length === 0) return null;
    if (autoPick && bucketFor(autoPick) === bucket) return autoPick;
    return inBucket.sort((a, b) => modelMemoryGb(a) - modelMemoryGb(b))[0];
  }

  // Default the user's pick to whichever bucket the auto-recommendation
  // lives in. The user can override; the override id is what gets
  // persisted as `backend.model` after apply.
  const autoBucket = $derived<SizeKey>(autoPick ? bucketFor(autoPick) : "medium");
  let activeBucket = $state<SizeKey | null>(null);
  const effectiveBucket = $derived<SizeKey>(activeBucket ?? autoBucket);
  const activeModel = $derived<RecommendedModelEntry | null>(
    byBucket[effectiveBucket] ?? autoPick,
  );

  // Push the active id into the bindable prop whenever it changes.
  $effect(() => {
    selectedModelId = activeModel ? modelKey(activeModel) : null;
  });

  function acceleratorLabel(): string {
    if (payload?.user?.detected_accelerator) return payload.user.detected_accelerator;
    if (hardware?.gpu_devices?.length) {
      return hardware.gpu_devices
        .map((device) => {
          if (device.memory_kind === "vram" && device.memory_display_gb) {
            return `${device.name} (${device.memory_display_gb} GB VRAM)`;
          }
          if (device.memory_kind === "unified" && hardware.memory_display_gb) {
            return `${device.name} (${hardware.memory_display_gb} GB unified memory)`;
          }
          return device.name;
        })
        .join(", ");
    }
    if (hardware?.gpu && hardware.gpu !== "none") {
      const label = {
        nvidia: "NVIDIA GPU",
        amd: "AMD GPU",
        apple_silicon: "Apple Silicon",
        integrated: "integrated graphics",
      }[hardware.gpu];
      const memory = hardware.gpu_vram_gb
        ? ` (${hardware.gpu_vram_gb} GB VRAM)`
        : hardware.memory_is_unified && hardware.memory_display_gb
          ? ` (${hardware.memory_display_gb} GB unified memory)`
          : "";
      return `${label}${memory}`;
    }
    if (hardware?.memory_display_gb) return `CPU (${hardware.memory_display_gb} GB system memory)`;
    if (hardware?.ram_gb) return `CPU (${hardware.ram_gb} GiB system memory)`;
    return "This computer";
  }

  function modelCaption(m: RecommendedModelEntry | null): string {
    if (!m) return "";
    const name = m.display_name ?? m.family ?? m.id ?? "Vaner local model";
    const id = m.model_id ?? m.id ?? "";
    const runtimeLabel = runtime?.label ?? m.runtime_label ?? m.runtime ?? "local runner";
    const idHint = id && id !== name ? ` · ${id}` : "";
    return `${runtimeLabel} · ${name}${idHint}`;
  }

  const SIZE_HINTS: Record<SizeKey, { label: string; hint: string }> = {
    light: { label: "Light", hint: "≤12 GB" },
    medium: { label: "Medium", hint: "12–24 GB" },
    heavy: { label: "Heavy", hint: "24 GB+" },
  };
</script>

<section class="card">
  {#if loading}
    <div class="loading"><Spinner size={16} /><span>Checking this computer…</span></div>
  {:else}
    <div class="row">
      <span class="label">Computer</span>
      <strong>{acceleratorLabel()}</strong>
    </div>

    {#if allCandidates.length > 0}
      <div class="row">
        <span class="label">Model size</span>
        <div class="seg" role="radiogroup" aria-label="Local model size">
          {#each (Object.keys(SIZE_HINTS) as SizeKey[]) as bucket (bucket)}
            {@const enabled = Boolean(byBucket[bucket])}
            <button
              type="button"
              role="radio"
              class="seg-btn"
              class:on={enabled && effectiveBucket === bucket}
              aria-checked={enabled && effectiveBucket === bucket}
              disabled={!enabled}
              onclick={() => (activeBucket = bucket)}
            >
              <span>{SIZE_HINTS[bucket].label}</span>
              <span class="hint">{SIZE_HINTS[bucket].hint}</span>
            </button>
          {/each}
        </div>
        <span class="caption">{modelCaption(activeModel)}</span>
      </div>
    {:else}
      <V1Body
        muted
        text="Vaner will choose the safest local setup it can verify on this computer."
      />
    {/if}
  {/if}
</section>

<style>
  .card {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding: 16px 18px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    color: var(--vd-fg-1);
  }
  .loading {
    display: flex;
    align-items: center;
    gap: 10px;
    color: var(--vd-fg-2);
    font-size: 13px;
  }
  .row {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .label {
    font-size: 11px;
    color: var(--vd-fg-3);
    text-transform: uppercase;
    letter-spacing: 0.06em;
  }
  strong {
    font-size: 14px;
    line-height: 1.35;
    font-weight: 500;
    color: var(--vd-fg-1);
  }
  .seg {
    display: inline-flex;
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip, 8px);
    overflow: hidden;
    align-self: flex-start;
    margin-top: 2px;
  }
  .seg-btn {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    padding: 6px 14px;
    background: transparent;
    color: var(--vd-fg-2);
    border: none;
    border-right: 0.5px solid var(--vd-line);
    cursor: pointer;
    font: inherit;
    transition: background-color 120ms ease, color 120ms ease;
  }
  .seg-btn:last-child { border-right: none; }
  .seg-btn:hover:not(:disabled) { background: var(--vd-bg-2); }
  .seg-btn.on {
    background: color-mix(in srgb, var(--vd-amber) 16%, var(--vd-bg-1));
    color: var(--vd-fg-1);
  }
  .seg-btn:disabled { opacity: 0.3; cursor: not-allowed; }
  .seg-btn .hint {
    font-size: 10px;
    color: var(--vd-fg-3);
  }
  .caption {
    font-size: 11px;
    color: var(--vd-fg-3);
    font-family: var(--vd-font-mono, monospace);
    margin-top: 6px;
  }
</style>
