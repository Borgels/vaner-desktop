<!--
  SetupWizard — the multi-slide setup flow used by both the dedicated
  onboarding window (/onboarding) and the in-app re-runnable wizard
  route (/setup).

  Format: one focused slide per question, vertically centered inside a
  720×540 (or larger) container. Step dots at the top. Back / Next /
  Skip controls at the bottom.

  Slides:
    0  Welcome (brand + tagline)
    1  Work styles    (multi-select chips)
    2  Priority       (single-select chips)
    3  Compute posture
    4  Background posture
    5  Recommendation review (bundle id, tier, "why this bundle?")
    6  Apply + Done   (closing slide)

  Cloud posture is intentionally NOT asked during onboarding. It needs
  API-key configuration the wizard doesn't perform, and cloud-LLM cost
  dynamics can sting users who don't know which provider Vaner would
  call. The wizard always submits `local_only`; users opt in via
  Preferences (Companion → Preferences) where the API-key flow is
  paired with the toggle. The widening dance below is dead code in the
  default flow but kept as a safety net in case the recommended bundle
  itself proposes a wider cloud_posture than the previous policy.

  `onComplete` is called after a successful apply — the parent decides
  what that means (close the onboarding window, or goto('/') back to
  the popover).
-->
<script lang="ts">
  import { onMount } from "svelte";
  import {
    setup,
    loadQuestions,
    loadStatus,
    loadHardware,
    loadModelRecommendation,
    recommend,
    apply,
    type ModelsRecommendedPayload,
  } from "$lib/stores/setup.js";
  import { showToast } from "$lib/stores/toast.js";
  import VMark from "$lib/components/primitives/VMark.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import RecommendedPresetCard from "$lib/components/RecommendedPresetCard.svelte";
  import type {
    BackgroundPosture,
    CloudPosture,
    ComputePosture,
    Priority,
    SelectionResult,
    SetupAnswers,
    SetupQuestion,
    WorkStyle,
  } from "$lib/contract/setup-types.js";

  type Props = {
    /** Called after a successful setup_apply (post cloud-widening
     *  confirm if applicable). The parent dispatches the right
     *  exit behavior — close the onboarding window, or goto('/'). */
    onComplete: () => void | Promise<void>;
    /** Called when the user dismisses the wizard. */
    onSkip: () => void | Promise<void>;
  };
  const { onComplete, onSkip }: Props = $props();

  const TOTAL_SLIDES = 7;
  let slide = $state(0);

  let questions = $state<SetupQuestion[]>([]);
  let workStyles = $state<WorkStyle[]>(["mixed"]);
  let priority = $state<Priority>("balanced");
  let computePosture = $state<ComputePosture>("balanced");
  // Cloud posture is hidden from the wizard — see header comment.
  // Hardcoded to `local_only`; cloud opt-in lives in Preferences.
  const CLOUD_POSTURE_DEFAULT: CloudPosture = "local_only";
  let backgroundPosture = $state<BackgroundPosture>("normal");
  let recommendation = $state<SelectionResult | null>(null);
  let recommending = $state(false);
  let modelRecommendation = $state<ModelsRecommendedPayload | null>(null);
  let modelRecommending = $state(false);
  let applying = $state(false);
  let widening = $state<{ id: string; reasons: string[] } | null>(null);
  let appliedBundleId = $state<string | null>(null);

  const hardware = $derived($setup.hardware);

  onMount(async () => {
    questions = await loadQuestions();
    await loadStatus();
    await loadHardware();
  });

  function getQuestion(id: string): SetupQuestion | undefined {
    return questions.find((q) => q.id === id);
  }

  function answers(): SetupAnswers {
    return {
      work_styles: workStyles.length === 0 ? ["mixed"] : workStyles,
      priority,
      compute_posture: computePosture,
      cloud_posture: CLOUD_POSTURE_DEFAULT,
      background_posture: backgroundPosture,
    };
  }

  function toggleWorkStyle(v: WorkStyle) {
    workStyles = workStyles.includes(v)
      ? workStyles.filter((x) => x !== v)
      : [...workStyles, v];
  }

  async function nextSlide() {
    if (slide === 4) {
      // Going from background-posture (slide 4) into Recommendation
      // (slide 5) requires a network round-trip — fetch both the
      // bundle selection AND the hardware-driven model recommendation
      // in parallel so the preset card on slide 5 doesn't show a
      // second spinner.
      recommending = true;
      modelRecommending = true;
      try {
        const [sel, modelRec] = await Promise.all([
          recommend(answers()),
          loadModelRecommendation(workStyles),
        ]);
        recommendation = sel;
        modelRecommendation = modelRec;
        if (recommendation) slide = 5;
      } finally {
        recommending = false;
        modelRecommending = false;
      }
      return;
    }
    if (slide === 5) {
      // From Recommendation review → Apply slide.
      slide = 6;
      void doApply();
      return;
    }
    slide = Math.min(slide + 1, TOTAL_SLIDES - 1);
  }

  function prevSlide() {
    if (slide > 0) slide -= 1;
  }

  async function doApply(confirmWidening = false) {
    applying = true;
    try {
      const result = await apply({
        answers: answers(),
        confirm_cloud_widening: confirmWidening,
      });
      if (!result) return;
      if (result.widens_cloud_posture && !result.written) {
        widening = {
          id: result.selected_bundle_id,
          reasons: result.reasons,
        };
        return;
      }
      widening = null;
      appliedBundleId = result.selected_bundle_id;
      showToast(`Setup complete: ${result.selected_bundle_id}`, "success", 3500);
      // Hold on the success slide for a beat, then hand off.
      setTimeout(() => {
        void onComplete();
      }, 900);
    } finally {
      applying = false;
    }
  }

  // Per-slide button labels.
  const nextLabel = $derived(
    slide === 0
      ? "Get started"
      : slide === 4
        ? recommending
          ? "Reading hardware…"
          : "See recommendation"
        : slide === 5
          ? "Apply"
          : "Continue",
  );
  const nextDisabled = $derived(
    (slide === 1 && workStyles.length === 0) || recommending || applying,
  );

  // Build each question's choice list lazily so the chips can read
  // `findChoiceLabel("work_styles", "coding")` style.
  function choices(qid: string) {
    return getQuestion(qid)?.choices ?? [];
  }
</script>

<div class="wizard">
  <!-- Step indicator -->
  <header class="dots">
    {#each Array.from({ length: TOTAL_SLIDES }) as _, i (i)}
      <span class="dot" class:active={slide >= i} class:current={slide === i}></span>
    {/each}
  </header>

  <main class="slide-stage">
    {#if slide === 0}
      <!-- 0 · Welcome -->
      <section class="slide welcome">
        <VMark size={48} satelliteState="prepared" breathing />
        <V1Kicker text="Welcome" />
        <h1>A quiet companion that thinks ahead.</h1>
        <p class="lead">
          Five quick questions, then Vaner picks a profile that matches your
          machine and how you work. Reversible from Preferences any time.
        </p>
      </section>
    {:else if slide === 1}
      <!-- 1 · Work styles -->
      <section class="slide">
        <V1Kicker text={`Question 1 of 4`} />
        <h1>{getQuestion("work_styles")?.prompt ?? "What kinds of work?"}</h1>
        <p class="lead">Pick all that apply.</p>
        <div class="chips multi">
          {#each choices("work_styles") as c (c.value)}
            <button
              type="button"
              class="chip"
              class:on={workStyles.includes(c.value as WorkStyle)}
              onclick={() => toggleWorkStyle(c.value as WorkStyle)}
            >
              <span>{c.label}</span>
              {#if c.hint}
                <span class="hint">{c.hint}</span>
              {/if}
            </button>
          {/each}
        </div>
      </section>
    {:else if slide === 2}
      <!-- 2 · Priority -->
      <section class="slide">
        <V1Kicker text={`Question 2 of 4`} />
        <h1>{getQuestion("priority")?.prompt ?? "What matters most?"}</h1>
        <div class="chips single">
          {#each choices("priority") as c (c.value)}
            <button
              type="button"
              class="chip"
              class:on={priority === c.value}
              onclick={() => (priority = c.value as Priority)}
            >
              <span>{c.label}</span>
              {#if c.hint}<span class="hint">{c.hint}</span>{/if}
            </button>
          {/each}
        </div>
      </section>
    {:else if slide === 3}
      <!-- 3 · Compute posture -->
      <section class="slide">
        <V1Kicker text={`Question 3 of 4`} />
        <h1>{getQuestion("compute_posture")?.prompt ?? "How hard should Vaner work?"}</h1>
        <div class="chips single">
          {#each choices("compute_posture") as c (c.value)}
            <button
              type="button"
              class="chip"
              class:on={computePosture === c.value}
              onclick={() => (computePosture = c.value as ComputePosture)}
            >
              <span>{c.label}</span>
              {#if c.hint}<span class="hint">{c.hint}</span>{/if}
            </button>
          {/each}
        </div>
      </section>
    {:else if slide === 4}
      <!-- 4 · Background posture (cloud-posture intentionally NOT asked
           during onboarding — see CLOUD_POSTURE_DEFAULT comment in script;
           opt-in lives in Preferences, paired with API-key setup) -->
      <section class="slide">
        <V1Kicker text={`Question 4 of 4`} />
        <h1>{getQuestion("background_posture")?.prompt ?? "How busy should Vaner be in the background?"}</h1>
        <div class="chips single">
          {#each choices("background_posture") as c (c.value)}
            <button
              type="button"
              class="chip"
              class:on={backgroundPosture === c.value}
              onclick={() => (backgroundPosture = c.value as BackgroundPosture)}
            >
              <span>{c.label}</span>
              {#if c.hint}<span class="hint">{c.hint}</span>{/if}
            </button>
          {/each}
        </div>
      </section>
    {:else if slide === 5}
      <!-- 5 · Recommendation review -->
      <section class="slide review">
        <V1Kicker text="Recommended bundle" color="var(--vd-amber)" />
        {#if recommendation}
          <h1>{recommendation.bundle.label}</h1>
          <p class="bundle-desc">{recommendation.bundle.description}</p>
          {#if hardware?.tier}
            <p class="tier-badge">Hardware tier · {hardware.tier}</p>
          {/if}
          {#if recommendation.reasons?.length}
            <ul class="reasons">
              {#each recommendation.reasons as r (r)}
                <li>
                  <span class="bullet"></span>
                  <span>{r}</span>
                </li>
              {/each}
            </ul>
          {/if}
          <RecommendedPresetCard payload={modelRecommendation} loading={modelRecommending} />
        {:else}
          <div class="loading"><Spinner size={20} /><span>Picking a bundle…</span></div>
        {/if}
      </section>
    {:else}
      <!-- 6 · Apply / Done. Widening branch is mostly dead code now
           that the onboarding wizard always submits local_only — but
           kept as a safety net in case the recommended bundle itself
           has a wider local_cloud_posture than the previous policy. -->
      <section class="slide done">
        {#if widening}
          <V1Kicker text="One more thing" color="var(--vd-amber)" />
          <h1>The recommended bundle would widen cloud access.</h1>
          <p class="lead">
            <strong>{widening.id}</strong> proposes a wider cloud posture
            than your current policy. The wizard normally submits
            <em>local_only</em> — pick how to proceed.
          </p>
          <div class="actions inline">
            <V1PrimaryButton title="Allow widening" tint="var(--vd-amber)" onclick={() => doApply(true)} />
            <V1GhostButton title="Keep local-only" onclick={() => { widening = null; slide = 5; }} />
          </div>
        {:else if applying}
          <div class="loading"><Spinner size={20} /><span>Saving…</span></div>
        {:else if appliedBundleId}
          <VMark size={48} satelliteState="prepared" />
          <V1Kicker text="All set" color="var(--vd-st-on)" />
          <h1>Vaner is ready.</h1>
          <p class="lead">
            Bundle <strong>{appliedBundleId}</strong> active. The popover
            will start surfacing prepared moments as Vaner reads your work.
          </p>
        {:else}
          <div class="loading"><Spinner size={20} /><span>Saving…</span></div>
        {/if}
      </section>
    {/if}
  </main>

  <!-- Footer controls -->
  <footer class="ctl">
    <V1GhostButton title="Skip for now" onclick={() => onSkip()} />
    <span class="spacer"></span>
    {#if slide > 0 && slide < 7}
      <V1GhostButton title="Back" onclick={prevSlide} disabled={recommending || applying} />
    {/if}
    {#if slide < 7}
      <V1PrimaryButton
        title={nextLabel}
        tint={slide === 6 ? "var(--vd-amber)" : undefined}
        disabled={nextDisabled}
        onclick={nextSlide}
      />
    {/if}
  </footer>
</div>

<style>
  .wizard {
    display: flex;
    flex-direction: column;
    height: 100vh;
    background: var(--vd-bg-0);
    color: var(--vd-fg-1);
    font-family: var(--vd-font);
    padding: 22px 36px 22px;
    overflow: hidden;
  }
  .dots {
    display: flex;
    gap: 6px;
    align-items: center;
    flex: 0 0 auto;
  }
  .dot {
    width: 6px;
    height: 6px;
    border-radius: 50%;
    background: var(--vd-line);
    transition: background 0.18s ease-out, transform 0.18s ease-out;
  }
  .dot.active {
    background: color-mix(in srgb, var(--vd-amber) 60%, var(--vd-fg-3));
  }
  .dot.current {
    background: var(--vd-amber);
    transform: scale(1.4);
  }

  .slide-stage {
    flex: 1 1 auto;
    display: flex;
    /* Top-align so longer slides (e.g. 8-option work_styles) don't
       push their h1 above the dots row. flex-start prevents the
       overlap; overflow-y: auto lets the slide scroll if the content
       still doesn't fit (rare). */
    align-items: flex-start;
    justify-content: center;
    overflow-y: auto;
    overflow-x: hidden;
    padding: 16px 0 12px;
    min-height: 0;
    scrollbar-width: thin;
    scrollbar-color: var(--vd-line) transparent;
  }
  .slide-stage::-webkit-scrollbar {
    width: 6px;
  }
  .slide-stage::-webkit-scrollbar-thumb {
    background: var(--vd-line);
    border-radius: 3px;
  }
  .slide {
    max-width: 540px;
    width: 100%;
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .slide.welcome,
  .slide.done {
    gap: 14px;
    margin-top: 8px;
  }
  .slide h1 {
    margin: 2px 0 0;
    font-family: var(--vd-font);
    font-size: 22px;
    font-weight: 500;
    line-height: 1.22;
    letter-spacing: -0.014em;
    color: var(--vd-fg-1);
  }
  .slide .lead {
    margin: 0;
    font-size: 13px;
    color: var(--vd-fg-2);
    line-height: 1.55;
  }
  .slide .lead strong { font-weight: 500; color: var(--vd-fg-1); }
  .slide .lead em { font-style: italic; color: var(--vd-amber); }

  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
    margin-top: 4px;
  }
  /* Multi-select (work_styles, 8 options) — pill-style chips that
     pack horizontally. The hint line is hidden on multi to keep the
     chip compact; macOS shows just the label on this question too. */
  .chips.multi {
    flex-direction: row;
  }
  .chips.multi .chip {
    flex: 0 0 auto;
    padding: 7px 12px;
    font-size: 12px;
    line-height: 1.25;
  }
  .chips.multi .chip .hint {
    display: none;
  }
  /* Single-select (priority, postures — 3-4 options each) — full-width
     stacked rows with the descriptive hint visible. */
  .chips.single {
    flex-direction: column;
    align-items: stretch;
  }
  .chip {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
    padding: 9px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
    color: var(--vd-fg-1);
    font-family: var(--vd-font);
    font-size: 13px;
    cursor: pointer;
    text-align: left;
    transition: background 0.12s, border-color 0.12s;
  }
  .chip:hover {
    background: var(--vd-bg-2);
  }
  .chip.on {
    background: color-mix(in srgb, var(--vd-amber) 14%, transparent);
    border-color: color-mix(in srgb, var(--vd-amber) 50%, transparent);
  }
  .chip .hint {
    font-size: 11px;
    color: var(--vd-fg-3);
  }
  .chip.on .hint {
    color: var(--vd-fg-2);
  }

  /* Review slide */
  .bundle-desc {
    margin: 0;
    font-size: 13px;
    color: var(--vd-fg-2);
    line-height: 1.55;
  }
  .tier-badge {
    margin: 0;
    font-family: var(--vd-font-mono);
    font-size: 11px;
    color: var(--vd-fg-3);
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }
  .reasons {
    list-style: none;
    margin: 8px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .reasons li {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    font-size: 13px;
    color: var(--vd-fg-2);
    line-height: 1.5;
  }
  .reasons .bullet {
    width: 5px;
    height: 5px;
    margin-top: 8px;
    border-radius: 50%;
    background: var(--vd-amber);
    flex: 0 0 auto;
  }

  .loading {
    display: inline-flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    color: var(--vd-fg-2);
  }
  .actions.inline {
    display: flex;
    gap: 8px;
    margin-top: 6px;
    flex-wrap: wrap;
  }

  .ctl {
    flex: 0 0 auto;
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 12px;
    border-top: 0.5px solid var(--vd-hair);
  }
  .ctl .spacer {
    flex: 1 1 auto;
  }
</style>
