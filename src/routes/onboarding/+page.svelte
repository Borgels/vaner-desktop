<!--
  Onboarding welcome screen. The first thing a user sees on first launch
  (Tauri opens this window when setup_status.completed_at is null). One
  page, three beats: brand intro → what Vaner does → "Get started" button
  that navigates the same window to /setup. The setup wizard's apply step
  detects it's running inside the onboarding window (label === "onboarding")
  and closes the window via close_onboarding instead of redirecting.

  Structurally simpler than the macOS 6-step OnboardingWindow but the same
  outcome: configured engine + first source connected, popover ready to
  surface real moments. Steps 5/6 of the macOS flow (connect-source +
  permissions request) are folded into the existing setup wizard +
  FirstRunGuidance modal in the popover.
-->
<script lang="ts">
  import { goto } from "$app/navigation";
  import VMark from "$lib/components/primitives/VMark.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import VStateBadge from "$lib/components/primitives/VStateBadge.svelte";

  function getStarted() {
    goto("/setup");
  }
</script>

<svelte:head>
  <title>Welcome to Vaner</title>
</svelte:head>

<div class="onboarding">
  <header class="brand">
    <VMark size={36} satelliteState="prepared" breathing />
    <span class="wordmark">vaner<span class="cursor">_</span></span>
  </header>

  <main class="hero">
    <V1Kicker text="Welcome" />
    <h1 class="title">A quiet companion that thinks ahead.</h1>
    <V1Body
      muted
      text={`Vaner reads your work — code, notes, calendar, conversations — and prepares context for the AI agents you already use. It runs locally, stays silent by default, and only speaks up when something is worth your attention.`}
    />

    <div class="bullets">
      <div class="bullet">
        <VStateBadge state="learning" size={8} />
        <div>
          <strong>Reads what you read.</strong>
          <span>Index your sources once and keep working — Vaner learns from your work in the background.</span>
        </div>
      </div>
      <div class="bullet">
        <VStateBadge state="prepared" size={8} />
        <div>
          <strong>Prepares context for the next prompt.</strong>
          <span>When something is worth surfacing, Vaner has the right files, threads, and history bundled up and ready to send.</span>
        </div>
      </div>
      <div class="bullet">
        <VStateBadge state="active" size={8} />
        <div>
          <strong>Works with whichever agent you're already using.</strong>
          <span>Cursor, Claude Desktop, Zed, VS Code, Continue — all reach Vaner over MCP. No lock-in.</span>
        </div>
      </div>
    </div>
  </main>

  <footer class="cta">
    <V1Body muted text="Setup is short and reversible. You can change anything later from the companion window." />
    <div class="actions">
      <V1PrimaryButton title="Get started" tint="var(--vd-amber)" onclick={getStarted} />
      <V1GhostButton title="Skip for now" onclick={() => window.close()} />
    </div>
  </footer>
</div>

<style>
  .onboarding {
    height: 100vh;
    display: flex;
    flex-direction: column;
    background: var(--vd-bg-0);
    color: var(--vd-fg-1);
    padding: 36px 56px 32px;
    font-family: var(--vd-font);
  }
  .brand {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .wordmark {
    font-family: var(--vd-font-term);
    font-size: 18px;
    color: var(--vd-fg-1);
    letter-spacing: 0.02em;
  }
  .cursor {
    color: var(--vd-amber);
    animation: vd-blink 1.6s cubic-bezier(0.4, 0, 0.2, 1) infinite;
  }
  .hero {
    flex: 1 1 auto;
    margin-top: 28px;
    max-width: 540px;
  }
  .title {
    margin: 8px 0 14px;
    font-family: var(--vd-font);
    font-size: 28px;
    font-weight: 500;
    letter-spacing: -0.018em;
    line-height: 1.18;
    color: var(--vd-fg-1);
  }
  .bullets {
    margin-top: 26px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .bullet {
    display: flex;
    gap: 14px;
    align-items: flex-start;
  }
  .bullet > div {
    display: flex;
    flex-direction: column;
    gap: 3px;
  }
  .bullet strong {
    font-weight: 500;
    font-size: 13.5px;
    color: var(--vd-fg-1);
  }
  .bullet span {
    font-size: 12.5px;
    color: var(--vd-fg-2);
    line-height: 1.5;
  }
  .cta {
    display: flex;
    flex-direction: column;
    gap: 12px;
    padding-top: 16px;
    border-top: 0.5px solid var(--vd-hair);
  }
  .actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }
</style>
