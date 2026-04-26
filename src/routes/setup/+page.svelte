<script lang="ts" context="module">
  function tierLabel(tier: string): string {
    switch (tier) {
      case "light":
        return "Light";
      case "capable":
        return "Capable";
      case "high_performance":
        return "High-performance";
      case "unknown":
        return "Unknown";
      default:
        return tier;
    }
  }
</script>

<script lang="ts">
  import { onMount } from "svelte";
  import { goto } from "$app/navigation";
  import {
    setup,
    loadStatus,
    loadQuestions,
    loadHardware,
    recommend,
    apply,
  } from "$lib/stores/setup.js";
  import { showToast } from "$lib/stores/toast.js";
  import type {
    BackgroundPosture,
    CloudPosture,
    ComputePosture,
    Priority,
    SetupAnswers,
    SetupQuestion,
    SelectionResult,
    WorkStyle,
  } from "$lib/contract/setup-types.js";

  // 0.8.6 WS-DESK-LINUX — first-run wizard.
  //
  // Five-step flow mirroring the macOS Onboarding/OnboardingWindow
  // five-step Simple-Mode flow:
  //   1. Welcome
  //   2. Work styles + priority
  //   3. Compute, cloud, background posture
  //   4. Hardware tier readout + bundle recommendation review
  //   5. Confirm + apply
  //
  // The wizard is dismissible (the user can hit "Cancel" any time);
  // the next app launch will re-fire the first-run check until apply
  // succeeds. The cloud-widening confirm dialog mirrors the macOS
  // pattern: if `setup_apply` returns `widens_cloud_posture=true,
  // written=false`, the dialog re-asks before re-calling with
  // confirm_cloud_widening=true.

  type Step = 1 | 2 | 3 | 4 | 5;

  let step: Step = 1;
  let questions: SetupQuestion[] = [];
  let workStyles: WorkStyle[] = ["mixed"];
  let priority: Priority = "balanced";
  let computePosture: ComputePosture = "balanced";
  let cloudPosture: CloudPosture = "ask_first";
  let backgroundPosture: BackgroundPosture = "normal";
  let recommendation: SelectionResult | null = null;
  let recommending = false;
  let applying = false;
  let widening: { selected_bundle_id: string; reasons: string[] } | null = null;

  onMount(async () => {
    questions = await loadQuestions();
    await loadStatus();
    await loadHardware();
  });

  function findChoiceLabel(qid: string, value: string): string {
    const q = questions.find((qq) => qq.id === qid);
    return q?.choices.find((c) => c.value === value)?.label ?? value;
  }

  function answers(): SetupAnswers {
    return {
      work_styles: workStyles.length === 0 ? ["mixed"] : workStyles,
      priority,
      compute_posture: computePosture,
      cloud_posture: cloudPosture,
      background_posture: backgroundPosture,
    };
  }

  function toggleWorkStyle(value: WorkStyle) {
    if (workStyles.includes(value)) {
      workStyles = workStyles.filter((s) => s !== value);
    } else {
      workStyles = [...workStyles, value];
    }
  }

  async function nextFromStep3() {
    recommending = true;
    try {
      recommendation = await recommend(answers());
      if (recommendation) {
        step = 4;
      }
    } finally {
      recommending = false;
    }
  }

  async function applyAndFinish(confirmWidening = false) {
    applying = true;
    try {
      const result = await apply({
        answers: answers(),
        confirm_cloud_widening: confirmWidening,
      });
      if (!result) {
        return;
      }
      if (result.widens_cloud_posture && !result.written) {
        widening = {
          selected_bundle_id: result.selected_bundle_id,
          reasons: result.reasons,
        };
        return;
      }
      widening = null;
      showToast(`Setup complete: ${result.selected_bundle_id}`, "success");
      await goto("/");
    } finally {
      applying = false;
    }
  }

  async function dismiss() {
    await goto("/");
  }

  $: hardware = $setup.hardware;
</script>

<div class="wizard">
  <header class="wizard-header">
    <div class="step-bar" aria-hidden="true">
      {#each [1, 2, 3, 4, 5] as i}
        <span class="dot" class:active={step >= (i as Step)} class:current={step === (i as Step)}></span>
      {/each}
    </div>
    <button class="cancel" type="button" on:click={dismiss}>Skip for now</button>
  </header>

  {#if step === 1}
    <section class="step welcome">
      <h1>Welcome to Vaner</h1>
      <p class="lead">Tell Vaner what you want help with.</p>
      <p>
        We'll ask five quick questions, take a look at this machine,
        and pick a profile that matches. You can change everything
        later from Preferences.
      </p>
      <div class="actions">
        <button class="primary" type="button" on:click={() => (step = 2)}>
          Get started
        </button>
      </div>
    </section>
  {:else if step === 2}
    <section class="step">
      <h1>What kind of work do you want help with?</h1>
      <p class="lead">Pick all that apply. Mixed is a safe default.</p>
      <div class="chips">
        {#each questions.find((q) => q.id === "work_styles")?.choices ?? [] as choice}
          <button
            type="button"
            class="chip"
            class:selected={workStyles.includes(choice.value as WorkStyle)}
            aria-pressed={workStyles.includes(choice.value as WorkStyle)}
            on:click={() => toggleWorkStyle(choice.value as WorkStyle)}
          >
            {choice.label}
          </button>
        {/each}
      </div>

      <h2>What matters most?</h2>
      <div class="choices">
        {#each questions.find((q) => q.id === "priority")?.choices ?? [] as choice}
          <label class="choice">
            <input
              type="radio"
              name="priority"
              value={choice.value}
              checked={priority === choice.value}
              on:change={() => (priority = choice.value as Priority)}
            />
            <span>{choice.label}</span>
          </label>
        {/each}
      </div>

      <div class="actions">
        <button type="button" on:click={() => (step = 1)}>Back</button>
        <button class="primary" type="button" on:click={() => (step = 3)}>
          Next
        </button>
      </div>
    </section>
  {:else if step === 3}
    <section class="step">
      <h1>How should Vaner behave on this machine?</h1>

      <h2>Compute posture</h2>
      <div class="choices">
        {#each questions.find((q) => q.id === "compute_posture")?.choices ?? [] as choice}
          <label class="choice">
            <input
              type="radio"
              name="compute_posture"
              value={choice.value}
              checked={computePosture === choice.value}
              on:change={() => (computePosture = choice.value as ComputePosture)}
            />
            <span>{choice.label}</span>
          </label>
        {/each}
      </div>

      <h2>Cloud posture</h2>
      <div class="choices">
        {#each questions.find((q) => q.id === "cloud_posture")?.choices ?? [] as choice}
          <label class="choice">
            <input
              type="radio"
              name="cloud_posture"
              value={choice.value}
              checked={cloudPosture === choice.value}
              on:change={() => (cloudPosture = choice.value as CloudPosture)}
            />
            <span>{choice.label}</span>
          </label>
        {/each}
      </div>

      <h2>Background pondering</h2>
      <div class="choices">
        {#each questions.find((q) => q.id === "background_posture")?.choices ?? [] as choice}
          <label class="choice">
            <input
              type="radio"
              name="background_posture"
              value={choice.value}
              checked={backgroundPosture === choice.value}
              on:change={() =>
                (backgroundPosture = choice.value as BackgroundPosture)}
            />
            <span>{choice.label}</span>
          </label>
        {/each}
      </div>

      <div class="actions">
        <button type="button" on:click={() => (step = 2)}>Back</button>
        <button
          class="primary"
          type="button"
          on:click={nextFromStep3}
          disabled={recommending}
        >
          {recommending ? "Recommending…" : "See recommendation"}
        </button>
      </div>
    </section>
  {:else if step === 4}
    <section class="step">
      <h1>Vaner suggests…</h1>

      {#if hardware}
        <div class="hardware-line">
          This machine looks like a
          <strong>{tierLabel(hardware.tier)}</strong>
          tier {hardware.os} box · {hardware.ram_gb} GB RAM ·
          {hardware.gpu === "none" ? "no GPU" : hardware.gpu}
          {#if hardware.detected_runtimes.length > 0}
            · runtimes: {hardware.detected_runtimes.join(", ")}
          {/if}
        </div>
      {/if}

      {#if recommendation}
        <div class="recommendation">
          <div class="bundle-label">{recommendation.bundle.label}</div>
          <div class="bundle-desc">{recommendation.bundle.description}</div>

          {#if recommendation.reasons.length > 0}
            <details open>
              <summary>Why this bundle?</summary>
              <ul>
                {#each recommendation.reasons as reason}
                  <li>{reason}</li>
                {/each}
              </ul>
            </details>
          {/if}

          {#if recommendation.runner_ups.length > 0}
            <details>
              <summary>Other candidates ({recommendation.runner_ups.length})</summary>
              <ul>
                {#each recommendation.runner_ups as r}
                  <li><strong>{r.label}</strong> — {r.description}</li>
                {/each}
              </ul>
            </details>
          {/if}

          {#if recommendation.forced_fallback}
            <p class="warning">
              ⚠ Filters narrowed the choices to none — Vaner fell back
              to the safest default. Review the candidates above.
            </p>
          {/if}
        </div>
      {:else}
        <p class="empty">No recommendation yet — go back and answer the questions.</p>
      {/if}

      <div class="actions">
        <button type="button" on:click={() => (step = 3)}>Back</button>
        <button
          class="primary"
          type="button"
          on:click={() => (step = 5)}
          disabled={!recommendation}
        >
          Looks good
        </button>
      </div>
    </section>
  {:else if step === 5}
    <section class="step">
      <h1>Confirm and apply</h1>
      <p class="lead">
        Vaner will write your answers to <code>.vaner/config.toml</code>
        and pick up the new bundle on the next engine cycle.
      </p>

      <div class="summary">
        <h2>Your answers</h2>
        <dl>
          <dt>Work styles</dt>
          <dd>{workStyles.map((s) => findChoiceLabel("work_styles", s)).join(", ")}</dd>
          <dt>Priority</dt>
          <dd>{findChoiceLabel("priority", priority)}</dd>
          <dt>Compute posture</dt>
          <dd>{findChoiceLabel("compute_posture", computePosture)}</dd>
          <dt>Cloud posture</dt>
          <dd>{findChoiceLabel("cloud_posture", cloudPosture)}</dd>
          <dt>Background posture</dt>
          <dd>{findChoiceLabel("background_posture", backgroundPosture)}</dd>
        </dl>
        {#if recommendation}
          <h2>Bundle</h2>
          <p class="bundle-label">{recommendation.bundle.label}</p>
        {/if}
      </div>

      <div class="actions">
        <button type="button" on:click={() => (step = 4)}>Back</button>
        <button
          class="primary"
          type="button"
          on:click={() => applyAndFinish(false)}
          disabled={applying}
        >
          {applying ? "Applying…" : "Apply and finish"}
        </button>
      </div>
    </section>
  {/if}

  {#if widening}
    <div class="dialog-backdrop" role="dialog" aria-modal="true" aria-labelledby="widening-title">
      <div class="dialog">
        <h2 id="widening-title">Widen cloud posture?</h2>
        <p>
          Selecting <strong>{widening.selected_bundle_id}</strong>
          relaxes your cloud posture compared to the current bundle.
          That means Vaner may reach for cloud LLMs more often.
        </p>
        {#if widening.reasons.length > 0}
          <ul class="reasons">
            {#each widening.reasons as reason}
              <li>{reason}</li>
            {/each}
          </ul>
        {/if}
        <p>Are you sure?</p>
        <div class="dialog-actions">
          <button type="button" on:click={() => (widening = null)}>Cancel</button>
          <button
            type="button"
            class="primary"
            on:click={() => applyAndFinish(true)}
            disabled={applying}
          >
            {applying ? "Applying…" : "Yes, widen and apply"}
          </button>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .wizard {
    max-width: 520px;
    margin: 0 auto;
    padding: 32px 24px;
    color: var(--vaner-fg, #f2f2f2);
    font-family: system-ui, -apple-system, sans-serif;
  }
  .wizard-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 24px;
  }
  .step-bar {
    display: flex;
    gap: 6px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--vaner-hair, #2a2a2a);
    display: inline-block;
  }
  .dot.active {
    background: var(--vaner-accent, #5eb2ff);
  }
  .dot.current {
    transform: scale(1.4);
  }
  .cancel {
    background: transparent;
    border: none;
    color: var(--vaner-muted, #888);
    cursor: pointer;
    font-size: 12px;
  }
  .step {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  h1 {
    font-size: 20px;
    font-weight: 600;
    margin: 0;
  }
  h2 {
    font-size: 13px;
    font-weight: 600;
    margin: 8px 0 4px;
    color: var(--vaner-muted, #888);
  }
  .lead {
    font-size: 14px;
    margin: 0 0 4px;
    color: var(--vaner-fg, #f2f2f2);
  }
  .chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    background: var(--vaner-bg-1, #181818);
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: var(--vaner-fg, #f2f2f2);
    padding: 6px 12px;
    border-radius: 999px;
    cursor: pointer;
    font-size: 12px;
  }
  .chip.selected {
    background: var(--vaner-accent, #5eb2ff);
    border-color: var(--vaner-accent, #5eb2ff);
    color: var(--vaner-bg-0, #111);
  }
  .choices {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .choice {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    font-size: 13px;
    padding: 4px 0;
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 16px;
  }
  .actions button {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    padding: 8px 16px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 13px;
    font-weight: 500;
  }
  .actions button.primary {
    background: var(--vaner-accent, #5eb2ff);
    border-color: var(--vaner-accent, #5eb2ff);
    color: var(--vaner-bg-0, #111);
  }
  .actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
  .hardware-line {
    font-size: 12px;
    color: var(--vaner-muted, #888);
    border-left: 3px solid var(--vaner-accent, #5eb2ff);
    padding: 6px 10px;
  }
  .recommendation {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    padding: 14px 16px;
    background: var(--vaner-bg-1, #181818);
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .bundle-label {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
  }
  .bundle-desc {
    font-size: 13px;
    color: var(--vaner-muted, #888);
    margin: 0;
  }
  details summary {
    cursor: pointer;
    font-size: 12px;
    color: var(--vaner-accent, #5eb2ff);
  }
  details ul {
    margin: 6px 0 0 18px;
    padding: 0;
    font-size: 12px;
    line-height: 1.5;
  }
  .warning {
    margin: 0;
    padding: 8px 10px;
    border-left: 3px solid #f5a524;
    background: rgba(245, 165, 36, 0.08);
    color: #f5a524;
    font-size: 12px;
  }
  .summary {
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 8px;
    padding: 14px 16px;
    background: var(--vaner-bg-1, #181818);
  }
  .summary dl {
    display: grid;
    grid-template-columns: max-content 1fr;
    column-gap: 18px;
    row-gap: 6px;
    margin: 0 0 12px;
    font-size: 12px;
  }
  .summary dt {
    color: var(--vaner-muted, #888);
  }
  .summary dd {
    margin: 0;
  }
  .empty {
    color: var(--vaner-muted, #888);
    font-size: 12px;
  }
  code {
    font-family: var(--vd-font-mono, ui-monospace, monospace);
    font-size: 11px;
    background: rgba(255, 255, 255, 0.06);
    padding: 1px 4px;
    border-radius: 3px;
  }

  /* Cloud-widening confirm dialog */
  .dialog-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }
  .dialog {
    max-width: 380px;
    padding: 22px 20px 18px;
    background: var(--vaner-bg-1, #181818);
    border: 1px solid var(--vaner-hair, #2a2a2a);
    border-radius: 10px;
  }
  .dialog h2 {
    margin: 0 0 10px;
    font-size: 14px;
    color: #f5a524;
  }
  .dialog p {
    font-size: 12px;
    color: var(--vaner-fg, #f2f2f2);
    line-height: 1.5;
    margin: 0 0 8px;
  }
  .dialog .reasons {
    margin: 6px 0 8px 18px;
    padding: 0;
    font-size: 11px;
    color: var(--vaner-muted, #888);
  }
  .dialog-actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    margin-top: 12px;
  }
  .dialog-actions button {
    background: transparent;
    border: 1px solid var(--vaner-hair, #2a2a2a);
    color: inherit;
    padding: 6px 14px;
    border-radius: 6px;
    cursor: pointer;
    font-size: 12px;
  }
  .dialog-actions button.primary {
    background: #f5a524;
    border-color: #f5a524;
    color: var(--vaner-bg-0, #111);
  }
  .dialog-actions button:disabled {
    opacity: 0.4;
    cursor: not-allowed;
  }
</style>
