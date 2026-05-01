<!--
  /dev/primitives — visual storyboard for every primitive.

  Open `pnpm tauri dev` (or http://localhost:1420/dev/primitives in a
  browser tab) to scan every primitive in isolation. Gated to
  development builds via the route name; nothing in the production
  popover ever links here.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import V1Slider from "$lib/components/primitives/V1Slider.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import VStateBadge from "$lib/components/primitives/VStateBadge.svelte";
  import VMark from "$lib/components/primitives/VMark.svelte";
  import VMenuBarIcon from "$lib/components/primitives/VMenuBarIcon.svelte";
  import Spinner from "$lib/components/primitives/Spinner.svelte";
  import SourceGlyph from "$lib/components/primitives/SourceGlyph.svelte";
  import VContextCard from "$lib/components/primitives/VContextCard.svelte";
  import VMenuRow from "$lib/components/primitives/VMenuRow.svelte";
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";

  let chattiness = $state(0.4);

  const states = ["on", "learning", "prepared", "active", "attention", "idle"] as const;
  const sources = [
    "github",
    "gitlab",
    "files",
    "linear",
    "slack",
    "calendar",
    "drive",
    "mail",
    "notion",
    "web",
    "code",
    "agent",
    "unknown",
  ];

  const leadMoment = {
    title: "Wire the cloud-widening confirm dialog",
    prediction:
      "When setup_apply returns widens_cloud_posture=true, re-call with confirm_cloud_widening=true after the user OKs.",
    sourceKind: "github",
    sourceLabel: "vaner-desktop · setup.rs",
    confidence: 0.82,
    kicker: "Prepared · 4m ago",
    reasons: [
      "You opened setup.rs three times today",
      "The endpoint contract is documented",
      "The macOS sibling already does this dance",
    ],
  };

  const supportingMoment = {
    title: "Read tray.rs",
    prediction: "The tray menu refresh + Pause defer landed in WS0.",
    sourceKind: "code",
    sourceLabel: "vaner-desktop · tray.rs",
    confidence: 0.61,
  };
</script>

<svelte:head>
  <title>vaner-desktop · primitives</title>
</svelte:head>

<div class="page vd-scroll">
  <h1 class="page-title">vaner-desktop · primitives</h1>
  <p class="page-sub">v0.2.2 storyboard. Every primitive in isolation; dark only.</p>

  <section>
    <VSectionLabel text="VMark · brand + states" />
    <div class="row">
      <VMark size={28} />
      <VMark size={28} satelliteState="on" />
      <VMark size={28} satelliteState="learning" breathing />
      <VMark size={28} satelliteState="prepared" />
      <VMark size={28} satelliteState="active" />
      <VMark size={28} satelliteState="attention" />
      <VMark size={28} satelliteState="idle" />
    </div>
    <div class="row">
      <VMark size={40} satelliteState="learning" breathing />
      <VMark size={56} satelliteState="prepared" />
      <VMenuBarIcon state="prepared" />
    </div>
  </section>

  <section>
    <VSectionLabel text="VStateBadge" />
    <div class="row">
      {#each states as s (s)}
        <span class="badge-cell">
          <VStateBadge state={s} />
          <span class="cell-label">{s}</span>
        </span>
      {/each}
    </div>
  </section>

  <section>
    <VSectionLabel text="Typography" />
    <V1Kicker text="Prepared · 4m ago" />
    <div style="height: 6px;"></div>
    <V1Headline text="Wire the cloud-widening confirm dialog" />
    <div style="height: 8px;"></div>
    <V1Body
      muted
      text="When setup_apply returns widens_cloud_posture=true, re-call with confirm_cloud_widening=true after the user OKs."
    />
  </section>

  <section>
    <VSectionLabel text="Buttons" />
    <div class="row">
      <V1PrimaryButton title="Send to agent" />
      <V1PrimaryButton title="Adopt" tint="var(--vd-amber)" />
      <V1PrimaryButton title="Disabled" disabled />
      <V1GhostButton title="Copy context" />
      <V1GhostButton title="Dismiss" destructive />
    </div>
  </section>

  <section>
    <VSectionLabel text="Slider · chattiness" />
    <V1Slider bind:value={chattiness} label="Chattiness ({(chattiness * 100).toFixed(0)}%)" />
  </section>

  <section>
    <VSectionLabel text="Spinner" />
    <div class="row">
      <Spinner size={12} />
      <Spinner size={18} />
      <Spinner size={32} />
    </div>
  </section>

  <section>
    <VSectionLabel text="SourceGlyph" />
    <div class="row source-grid">
      {#each sources as k (k)}
        <span class="badge-cell">
          <SourceGlyph kind={k} size={18} />
          <span class="cell-label">{k}</span>
        </span>
      {/each}
    </div>
  </section>

  <section>
    <VSectionLabel text="VContextCard · lead" />
    <div class="card-frame">
      <VContextCard moment={leadMoment} isLead />
    </div>
    <div style="height: 10px;"></div>
    <VSectionLabel text="VContextCard · supporting" />
    <div class="card-frame">
      <VContextCard moment={supportingMoment} />
    </div>
  </section>

  <section>
    <VSectionLabel text="VMenuRow" />
    <div class="card-frame menu-frame">
      <VMenuRow title="Prepared" detail="2" selected>
        {#snippet icon()}
          <SourceGlyph kind="agent" size={16} />
        {/snippet}
      </VMenuRow>
      <VMenuRow title="Sources" detail="6">
        {#snippet icon()}
          <SourceGlyph kind="github" size={16} />
        {/snippet}
      </VMenuRow>
      <VMenuRow title="Models" detail="4">
        {#snippet icon()}
          <SourceGlyph kind="code" size={16} />
        {/snippet}
      </VMenuRow>
    </div>
  </section>

  <section>
    <VSectionLabel text="QuietShell · scaffold" />
    <div class="popover-frame">
      <QuietShell markState="learning" breathingMark stateLabel="Learning · 2h">
        <V1Kicker text="Reading your work" />
        <div style="height: 6px;"></div>
        <V1Headline text="Nothing prepared yet — Vaner is indexing." />
        <div style="height: 8px;"></div>
        <V1Body
          muted
          text="Currently reading 12 files across 3 sources. Typically prepared in ~15m."
        />

        {#snippet footer()}
          <div class="footer-bits">
            <V1GhostButton title="Details" />
            <span class="footer-engine">
              <VStateBadge state="learning" size={6} />
              <span>Engine running</span>
            </span>
          </div>
        {/snippet}
      </QuietShell>
    </div>
  </section>
</div>

<style>
  .page {
    height: 100vh;
    padding: 28px 36px 80px;
    color: var(--vd-fg-1);
  }
  .page-title {
    font-family: var(--vd-font);
    font-size: 22px;
    font-weight: 500;
    margin: 0;
  }
  .page-sub {
    font-family: var(--vd-font);
    font-size: 12px;
    color: var(--vd-fg-3);
    margin: 4px 0 28px;
  }
  section {
    margin-top: 24px;
    max-width: 720px;
  }
  .row {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 16px;
    margin-top: 8px;
  }
  .source-grid {
    gap: 14px 24px;
  }
  .badge-cell {
    display: inline-flex;
    flex-direction: column;
    align-items: center;
    gap: 6px;
    width: 64px;
  }
  .cell-label {
    font-family: var(--vd-font);
    font-size: 10px;
    color: var(--vd-fg-3);
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }
  .card-frame {
    width: 360px;
    margin-top: 8px;
  }
  .menu-frame {
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    padding: 6px;
    width: 240px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .popover-frame {
    width: 420px;
    height: 360px;
    margin-top: 8px;
    background: var(--vd-bg-0);
    border-radius: var(--vd-r-pop);
    border: 0.5px solid var(--vd-line);
    box-shadow: var(--vd-shadow-pop);
    overflow: hidden;
    display: flex;
  }
  .popover-frame :global(.quiet-shell) {
    flex: 1 1 auto;
  }
  .footer-bits {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    color: var(--vd-fg-3);
    font-size: 11px;
    font-family: var(--vd-font);
  }
  .footer-engine {
    display: inline-flex;
    align-items: center;
    gap: 6px;
  }
</style>
