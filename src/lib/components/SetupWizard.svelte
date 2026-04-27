<!--
  SetupWizard — the multi-slide setup flow used by both the dedicated
  onboarding window (/onboarding) and the in-app re-runnable wizard
  route (/setup).

  Two flows on shared slide indices:

    Default (fast):
      0  Welcome
      1  Work styles                       (multi-select chips)
      4  Recommendation review + Apply     (preset card)
      5  Done

    Custom (Customize… button on slide 4):
      0  Welcome
      1  Work styles
      2  Priority                          (single-select chips, 5 choices)
      3  Energy                            (single-select; merges
                                             compute_posture + background_posture)
      4  Recommendation review + Apply
      5  Done

  Cloud posture is intentionally NOT asked during onboarding. It needs
  API-key configuration the wizard doesn't perform, and cloud-LLM cost
  dynamics can sting users who don't know which provider Vaner would
  call. The wizard always submits `local_only`; users opt in via
  Preferences (Companion → Preferences). The widening dance below is
  mostly dead code — kept as a safety net in case the recommended
  bundle itself proposes a wider cloud_posture than the existing
  policy.

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

  // Slide indices (shared by both flows; default skips 2 + 3):
  //   0 Welcome · 1 Work styles · 2 Priority · 3 Energy ·
  //   4 Recommendation review + Apply · 5 Done
  const TOTAL_SLIDES = 6;
  let slide = $state(0);

  // The wizard starts in the *default* (fast) flow. Clicking "Customize…"
  // on the recommendation review slide flips this on and rewinds to
  // slide 2 (Priority) so the user can fan out into the full question
  // set. Once on, it stays on for the session.
  let customMode = $state(false);

  let questions = $state<SetupQuestion[]>([]);
  let workStyles = $state<WorkStyle[]>(["mixed"]);
  let priority = $state<Priority>("balanced");
  // Energy collapses (compute_posture, background_posture) into one
  // user-facing question. Each value maps deterministically to a
  // (compute, background) pair. The daemon enums never see "energy";
  // we resolve this at submit time.
  type Energy = "light" | "balanced" | "burst" | "use_machine";
  let energy = $state<Energy>("balanced");
  const ENERGY_TO_POSTURES: Record<
    Energy,
    { compute: ComputePosture; background: BackgroundPosture }
  > = {
    light: { compute: "light", background: "minimal" },
    balanced: { compute: "balanced", background: "normal" },
    burst: { compute: "balanced", background: "idle_more" },
    use_machine: { compute: "available_power", background: "deep_run_aggressive" },
  };
  // Cloud posture is hidden from the wizard — see header comment.
  // Hardcoded to `local_only`; cloud opt-in lives in Preferences.
  const CLOUD_POSTURE_DEFAULT: CloudPosture = "local_only";
  let recommendation = $state<SelectionResult | null>(null);
  let recommending = $state(false);
  let modelRecommendation = $state<ModelsRecommendedPayload | null>(null);
  let modelRecommending = $state(false);
  let applying = $state(false);
  let widening = $state<{ id: string; reasons: string[] } | null>(null);
  let appliedBundleId = $state<string | null>(null);

  const hardware = $derived($setup.hardware);

  // Trim the daemon's option lists to the wizard's intended set.
  // The full enums stay valid on the daemon side — the in-app
  // /setup re-runnable surface and Preferences both still expose
  // every choice. The wizard hides the redundant ones.
  const WORK_STYLE_KEEP = new Set<WorkStyle>([
    "coding",
    "writing",
    "research",
    "planning",
    "support",
    "learning",
    "mixed",
  ]);
  const PRIORITY_KEEP = new Set<Priority>([
    "balanced",
    "speed",
    "quality",
    "privacy",
    "cost",
  ]);

  onMount(async () => {
    questions = await loadQuestions();
    await loadStatus();
    await loadHardware();
  });

  function getQuestion(id: string): SetupQuestion | undefined {
    return questions.find((q) => q.id === id);
  }

  function answers(): SetupAnswers {
    const postures = ENERGY_TO_POSTURES[energy];
    return {
      work_styles: workStyles.length === 0 ? ["mixed"] : workStyles,
      priority,
      compute_posture: postures.compute,
      cloud_posture: CLOUD_POSTURE_DEFAULT,
      background_posture: postures.background,
    };
  }

  function toggleWorkStyle(v: WorkStyle) {
    workStyles = workStyles.includes(v)
      ? workStyles.filter((x) => x !== v)
      : [...workStyles, v];
  }

  async function loadRecommendations() {
    recommending = true;
    modelRecommending = true;
    try {
      const [sel, modelRec] = await Promise.all([
        recommend(answers()),
        loadModelRecommendation(workStyles),
      ]);
      recommendation = sel;
      modelRecommendation = modelRec;
      return sel != null;
    } finally {
      recommending = false;
      modelRecommending = false;
    }
  }

  async function nextSlide() {
    // Slide 1 (Work styles): default flow skips Priority + Energy and
    // goes straight to the Recommendation review (slide 4); Custom
    // walks the full path.
    if (slide === 1) {
      if (customMode) {
        slide = 2;
        return;
      }
      const ok = await loadRecommendations();
      if (ok) slide = 4;
      return;
    }
    // Slide 3 (Energy, custom-only): same parallel-fetch transition into
    // the Recommendation review.
    if (slide === 3) {
      const ok = await loadRecommendations();
      if (ok) slide = 4;
      return;
    }
    // Slide 4 (Recommendation review): Apply.
    if (slide === 4) {
      slide = 5;
      void doApply();
      return;
    }
    slide = Math.min(slide + 1, TOTAL_SLIDES - 1);
  }

  function prevSlide() {
    if (slide === 0) return;
    // Default flow: from review (4) → Back lands on Work styles (1).
    if (slide === 4 && !customMode) {
      slide = 1;
      return;
    }
    slide -= 1;
  }

  function enterCustomFromReview() {
    customMode = true;
    slide = 2;
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
      : slide === 1
        ? customMode
          ? "Continue"
          : recommending
            ? "Reading hardware…"
            : "See recommendation"
        : slide === 3
          ? recommending
            ? "Reading hardware…"
            : "See recommendation"
          : slide === 4
            ? "Apply"
            : "Continue",
  );
  const nextDisabled = $derived(
    (slide === 1 && workStyles.length === 0) || recommending || applying,
  );

  // Build each question's choice list lazily so the chips can read
  // `findChoiceLabel("work_styles", "coding")` style. The wizard
  // filters the daemon's full option list down to the trimmed UI set
  // (see WORK_STYLE_KEEP / PRIORITY_KEEP at the top).
  function choices(qid: string) {
    return getQuestion(qid)?.choices ?? [];
  }
  function workStyleChoices() {
    return choices("work_styles").filter((c) =>
      WORK_STYLE_KEEP.has(c.value as WorkStyle),
    );
  }
  function priorityChoices() {
    return choices("priority").filter((c) => PRIORITY_KEEP.has(c.value as Priority));
  }

  // Energy choices are wizard-local — daemon does not have an "energy"
  // enum. Hard-coded labels here stay close to the macOS sibling.
  const ENERGY_CHOICES: Array<{ value: Energy; label: string; hint: string }> = [
    { value: "light", label: "Light", hint: "Barely use the CPU/GPU." },
    { value: "balanced", label: "Balanced", hint: "Work with what's idle (recommended)." },
    { value: "burst", label: "Burst when idle", hint: "Run broadly while the box is idle." },
    { value: "use_machine", label: "Use this machine", hint: "Cranked — happy to ponder overnight." },
  ];

  // Step dot count is mode-aware so the dots reflect the actual flow
  // length the user is walking through.
  const dotCount = $derived(customMode ? 6 : 4);
  // Visible-position-of-current-slide for the dots header.
  const dotIndex = $derived.by(() => {
    if (customMode) return slide;
    // Default flow only renders slides 0, 1, 4, 5 — collapse the gap.
    if (slide <= 1) return slide;
    if (slide === 4) return 2;
    if (slide >= 5) return 3;
    return slide;
  });
</script>

<div class="wizard">
  <!-- Step indicator (mode-aware: default = 4 dots, custom = 6 dots). -->
  <header class="dots">
    {#each Array.from({ length: dotCount }) as _, i (i)}
      <span class="dot" class:active={dotIndex >= i} class:current={dotIndex === i}></span>
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
          One question, then Vaner sizes your machine and picks a
          profile to match. Tweakable from Preferences any time.
        </p>
      </section>
    {:else if slide === 1}
      <!-- 1 · Work styles -->
      <section class="slide">
        <V1Kicker text={customMode ? "Question 1 of 3" : "What kind of work?"} />
        <h1>{getQuestion("work_styles")?.prompt ?? "What kinds of work?"}</h1>
        <p class="lead">Pick all that apply.</p>
        <div class="chips multi">
          {#each workStyleChoices() as c (c.value)}
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
      <!-- 2 · Priority (custom-mode only) -->
      <section class="slide">
        <V1Kicker text="Question 2 of 3" />
        <h1>{getQuestion("priority")?.prompt ?? "What matters most?"}</h1>
        <div class="chips single">
          {#each priorityChoices() as c (c.value)}
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
      <!-- 3 · Energy (custom-mode only; merges compute + background) -->
      <section class="slide">
        <V1Kicker text="Question 3 of 3" />
        <h1>How hard should Vaner work?</h1>
        <p class="lead">
          One knob covering both foreground compute and idle-time
          pondering. Pick a wider setting in Preferences if you want
          them split apart.
        </p>
        <div class="chips single">
          {#each ENERGY_CHOICES as c (c.value)}
            <button
              type="button"
              class="chip"
              class:on={energy === c.value}
              onclick={() => (energy = c.value)}
            >
              <span>{c.label}</span>
              <span class="hint">{c.hint}</span>
            </button>
          {/each}
        </div>
      </section>
    {:else if slide === 4}
      <!-- 4 · Recommendation review + Apply -->
      <section class="slide review">
        <V1Kicker text="Recommended for your machine" color="var(--vd-amber)" />
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
          {#if !customMode}
            <button type="button" class="customize-link" onclick={enterCustomFromReview}>
              Customize…
            </button>
          {/if}
        {:else}
          <div class="loading"><Spinner size={20} /><span>Picking a bundle…</span></div>
        {/if}
      </section>
    {:else}
      <!-- 5 · Apply / Done. Widening branch is mostly dead code now
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
            <V1GhostButton title="Keep local-only" onclick={() => { widening = null; slide = 4; }} />
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
    {#if slide > 0 && slide < TOTAL_SLIDES - 1}
      <V1GhostButton title="Back" onclick={prevSlide} disabled={recommending || applying} />
    {/if}
    {#if slide < TOTAL_SLIDES - 1}
      <V1PrimaryButton
        title={nextLabel}
        tint={slide === 4 ? "var(--vd-amber)" : undefined}
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
  .customize-link {
    background: none;
    border: none;
    padding: 0;
    margin: 8px 0 0;
    color: var(--vd-fg-2);
    font: inherit;
    font-size: 12px;
    cursor: pointer;
    align-self: flex-start;
  }
  .customize-link:hover {
    color: var(--vd-fg-1);
    text-decoration: underline;
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
