<!--
  Preferences pane — Linux equivalent of vaner-desktop-macos
  Companion/PreferencesPane.swift. Mirrors the macOS section structure:

    1. Active setup card
    2. Silent hours        ← toggle + from/to + weekdays-only
    3. Startup             ← Launch at login (Linux: XDG autostart)
    4. Memory              ← Export / Privacy / Clear

  Persona / tone sliders (chattiness / learnDepth / interrupt / voice)
  from seed.js were design speculation that never shipped on macOS.
  Removed.
-->
<script lang="ts">
  import { onMount } from "svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import VSectionLabel from "$lib/components/primitives/VSectionLabel.svelte";
  import { setup, loadStatus, loadHardware, loadPolicy } from "$lib/stores/setup.js";
  import { silentHours } from "$lib/stores/silent-hours.js";
  import { showToast } from "$lib/stores/toast.js";
  import ModelsCard from "$lib/components/ModelsCard.svelte";
  import {
    loadWorkspace,
    pickWorkspace,
    workspacePath,
  } from "$lib/stores/workspace.js";
  import {
    engineService,
    installEngineService,
    loadEngineServiceStatus,
    setEngineServiceLinger,
    uninstallEngineService,
    type ServiceState,
  } from "$lib/stores/engine-service.js";
  import {
    applyComputePreset,
    classifyPreset,
    computeConfig,
    loadComputeConfig,
    setComputeKey,
    type ComputePreset,
  } from "$lib/stores/compute-config.js";

  // Silent-hours window — From / To, weekdays-only. Persisted to
  // localStorage for v0.2.2 alongside the simple `silentHours` toggle
  // store. Daemon-side enforcement lands when the engine ships its
  // silent-hours endpoint.
  type SilentWindow = {
    startHour: number;
    endHour: number;
    weekdaysOnly: boolean;
  };
  const SILENT_KEY = "vaner.pref.silentWindow";
  const defaultsWindow: SilentWindow = {
    startHour: 9,
    endHour: 12,
    weekdaysOnly: true,
  };
  let silentWin = $state<SilentWindow>(loadSilent());
  function loadSilent(): SilentWindow {
    try {
      const raw = localStorage.getItem(SILENT_KEY);
      if (!raw) return { ...defaultsWindow };
      return { ...defaultsWindow, ...JSON.parse(raw) };
    } catch {
      return { ...defaultsWindow };
    }
  }
  $effect(() => {
    try {
      localStorage.setItem(SILENT_KEY, JSON.stringify(silentWin));
    } catch {
      /* localStorage unavailable */
    }
  });
  const fmtHour = (h: number) => `${String(h).padStart(2, "0")}:00`;

  // Launch-at-login on Linux is XDG autostart — we drop a .desktop
  // file at ~/.config/autostart/vaner-desktop.desktop. Toggle persists
  // to localStorage; actual file write is a v0.2.3 follow-up wiring.
  let launchAtLogin = $state<boolean>(
    (() => {
      try {
        return localStorage.getItem("vaner.pref.launchAtLogin") === "true";
      } catch {
        return false;
      }
    })(),
  );
  $effect(() => {
    try {
      localStorage.setItem("vaner.pref.launchAtLogin", String(launchAtLogin));
    } catch {
      /* noop */
    }
  });

  let confirmClear = $state(false);
  function exportMemory() {
    showToast("Memory export — daemon endpoint lands in v0.2.3.", "info", 3500);
  }
  function clearMemory() {
    confirmClear = false;
    showToast("Memory cleared on UI; daemon-side wipe pending v0.2.3.", "info", 3500);
  }

  let pickingWorkspace = $state(false);
  async function changeWorkspace() {
    if (pickingWorkspace) return;
    pickingWorkspace = true;
    try {
      const next = await pickWorkspace();
      if (next) {
        showToast(
          `Workspace set to ${next}. Restart the engine for the change to take effect.`,
          "success",
          5000,
        );
      }
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not change workspace: ${err}`,
        "attention",
        4000,
      );
    } finally {
      pickingWorkspace = false;
    }
  }

  // Performance card. Three preset rows + an Advanced disclosure
  // with the underlying knobs. Reads + writes through the Rust-side
  // engine_config commands; never touches .vaner/config.toml directly
  // so validation/aliases continue to live in the Python CLI.
  const PRESETS: { id: ComputePreset; title: string; subtitle: string }[] = [
    {
      id: "light",
      title: "Light",
      subtitle: "Quietest. CPU cap 15%, idle-only, scans cancel after 3 minutes.",
    },
    {
      id: "balanced",
      title: "Balanced",
      subtitle: "Default. CPU cap 25%, idle-only, scans cancel after 5 minutes.",
    },
    {
      id: "performance",
      title: "Performance",
      subtitle: "Loudest. CPU cap 50%, runs continuously, scans cancel after 10 minutes.",
    },
  ];

  let computeBusy = $state(false);
  let showAdvanced = $state(false);
  async function selectPreset(preset: ComputePreset) {
    if (computeBusy) return;
    computeBusy = true;
    try {
      await applyComputePreset(preset);
      showToast(`Performance set to ${preset}.`, "success", 2500);
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not apply preset: ${err}`,
        "attention",
        4000,
      );
    } finally {
      computeBusy = false;
    }
  }
  async function setKey(key: string, value: string) {
    if (computeBusy) return;
    computeBusy = true;
    try {
      await setComputeKey(key, value);
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not set ${key}: ${err}`,
        "attention",
        4000,
      );
      // Re-sync from disk in case the partial write left state out
      // of step with what the slider thinks.
      await loadComputeConfig();
    } finally {
      computeBusy = false;
    }
  }
  const activePreset = $derived(classifyPreset($computeConfig));

  let serviceBusy = $state(false);
  function describeServiceState(state: ServiceState | undefined): string {
    switch (state) {
      case "active":
        return "Running in the background. Survives desktop close + login restart.";
      case "enabled":
        return "Enabled but not currently running. systemd will start it on next login.";
      case "disabled":
        return "Unit installed but disabled. Toggle on to bring it up.";
      case "missing":
        return "Not installed. Toggle on to install + enable + start the unit.";
      case "unavailable":
        return "systemctl --user is unavailable on this session — enable Linger or use the auto-start fallback while the desktop is open.";
      default:
        return "Checking…";
    }
  }
  async function onServiceToggleClick(target: boolean) {
    if (serviceBusy) return;
    serviceBusy = true;
    try {
      if (target) {
        const status = await installEngineService();
        showToast(
          status.state === "active" ? "Background engine service started." : "Background engine service installed.",
          "success",
          3500,
        );
      } else {
        await uninstallEngineService();
        showToast("Background engine service stopped + removed.", "success", 3000);
      }
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Service action failed: ${err}`,
        "attention",
        5000,
      );
      // Re-sync from disk in case install partially succeeded.
      await loadEngineServiceStatus();
    } finally {
      serviceBusy = false;
    }
  }

  let lingerBusy = $state(false);
  async function onLingerToggleClick(target: boolean) {
    if (lingerBusy) return;
    lingerBusy = true;
    try {
      const status = await setEngineServiceLinger(target);
      showToast(
        status.linger_enabled
          ? "Linger enabled — the engine will keep running across logout."
          : "Linger disabled — the engine will stop on logout.",
        "success",
        3500,
      );
    } catch (err) {
      const msg = err instanceof Error ? err.message : `Linger toggle failed: ${err}`;
      showToast(msg, "attention", 5000);
      // pkexec may have changed nothing on cancel — re-sync to be sure.
      await loadEngineServiceStatus();
    } finally {
      lingerBusy = false;
    }
  }

  onMount(() => {
    loadStatus();
    loadHardware();
    loadPolicy();
    // The popover hydrates the workspace store on its own; the
    // companion window mounts independently, so hydrate here too
    // to surface the actual current path instead of "(none)".
    loadWorkspace();
    loadEngineServiceStatus();
    loadComputeConfig();
  });

  const bundle = $derived($setup.bundle);
  const tier = $derived($setup.hardware?.tier);
</script>

<header class="hd">
  <V1Kicker text="Preferences" />
  <V1Headline text="How Vaner sounds and when it speaks" size={22} />
</header>

<!-- Active bundle -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-purple);"></span><span>Active setup</span></div>
  {#if bundle}
    <div class="bundle-row">
      <span class="bundle-name">{bundle.id ?? "—"}</span>
      {#if tier}
        <span class="tier">{tier}</span>
      {/if}
    </div>
    {#if bundle.description}
      <p class="muted">{bundle.description}</p>
    {/if}
    <div class="actions">
      <V1GhostButton title="Re-run setup wizard" onclick={() => (window.location.href = "/setup")} />
    </div>
  {:else}
    <p class="muted">Loading bundle…</p>
  {/if}
</div>

<!-- Workspace -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-amber);"></span><span>Workspace</span></div>
  <V1Body
    muted
    text="The repository Vaner watches and indexes. Every CLI invocation (vaner up / status / setup) targets this path."
  />
  <div class="workspace-row">
    {#if $workspacePath}
      <code class="workspace-path">{$workspacePath}</code>
    {:else}
      <span class="workspace-empty">No workspace selected.</span>
    {/if}
  </div>
  <div class="actions">
    <V1GhostButton
      title={pickingWorkspace ? "Opening…" : $workspacePath ? "Change…" : "Pick a folder…"}
      onclick={changeWorkspace}
    />
  </div>
</div>

<!-- Models (Ollama) -->
<ModelsCard />

<!-- Silent hours -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-fg-3);"></span><span>Silent hours</span></div>
  <label class="row">
    <input type="checkbox" bind:checked={$silentHours} />
    <span class="row-text">
      <span class="row-title">Suppress interrupts during deep work</span>
      <span class="row-detail">Vaner still indexes; it just doesn't surface anything until silent hours end.</span>
    </span>
  </label>

  <div class="window" class:dim={!$silentHours}>
    <span class="window-label">From</span>
    <button class="step" onclick={() => silentWin.startHour = (silentWin.startHour + 23) % 24} disabled={!$silentHours}>−</button>
    <span class="window-time">{fmtHour(silentWin.startHour)}</span>
    <button class="step" onclick={() => silentWin.startHour = (silentWin.startHour + 1) % 24} disabled={!$silentHours}>+</button>

    <span class="window-label">to</span>
    <button class="step" onclick={() => silentWin.endHour = (silentWin.endHour + 23) % 24} disabled={!$silentHours}>−</button>
    <span class="window-time">{fmtHour(silentWin.endHour)}</span>
    <button class="step" onclick={() => silentWin.endHour = (silentWin.endHour + 1) % 24} disabled={!$silentHours}>+</button>
  </div>

  <label class="row toggle-only" class:dim={!$silentHours}>
    <input type="checkbox" bind:checked={silentWin.weekdaysOnly} disabled={!$silentHours} />
    <span class="row-title">Weekdays only</span>
  </label>

  <p class="hint">Prepared moments are held silently — Vaner surfaces them when you're back.</p>
</div>

<!-- Performance -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-st-active);"></span><span>Performance</span></div>
  {#if $computeConfig}
    {@const cfg = $computeConfig}
    <V1Body
      muted
      text="How hard Vaner pushes the engine. Pick a preset for one-click defaults, or expand Advanced to nudge the knobs."
    />
    <div class="presets">
      {#each PRESETS as preset (preset.id)}
        {@const selected = activePreset === preset.id}
        <button
          type="button"
          class="preset-row"
          class:selected
          disabled={computeBusy}
          onclick={() => selectPreset(preset.id)}
        >
          <span class="preset-title">{preset.title}</span>
          <span class="preset-sub">{preset.subtitle}</span>
          {#if selected}
            <span class="preset-badge">ACTIVE</span>
          {/if}
        </button>
      {/each}
    </div>

    <button
      type="button"
      class="adv-toggle"
      onclick={() => (showAdvanced = !showAdvanced)}
    >
      <span aria-hidden="true">{showAdvanced ? "▾" : "▸"}</span>
      <span>Advanced</span>
    </button>

    {#if showAdvanced}
      <div class="adv-block">
        <p class="adv-explainer">
          A <em>scan</em> is one pass where Vaner re-reads what's changed in your repo and updates its memory. Vaner runs scans continuously in the background.
        </p>

        <label class="slider-row">
          <span class="slider-label">
            CPU cap
            <span class="slider-value">{Math.round(cfg.cpu_fraction * 100)}%</span>
          </span>
          <input
            type="range"
            min="5"
            max="100"
            step="5"
            value={Math.round(cfg.cpu_fraction * 100)}
            disabled={computeBusy}
            onchange={(e) =>
              setKey("compute.cpu_fraction", String(Number((e.currentTarget as HTMLInputElement).value) / 100))}
          />
          <span class="slider-help">Maximum share of host CPU the scan loop may use.</span>
        </label>

        <label class="slider-row">
          <span class="slider-label">
            Cancel scan after
            <span class="slider-value">{cfg.max_cycle_seconds}s</span>
          </span>
          <input
            type="range"
            min="30"
            max="1800"
            step="30"
            value={cfg.max_cycle_seconds}
            disabled={computeBusy}
            onchange={(e) =>
              setKey("compute.max_cycle_seconds", (e.currentTarget as HTMLInputElement).value)}
          />
          <span class="slider-help">Hard ceiling on a single scan. Anything longer is rescheduled.</span>
        </label>

        <label class="row">
          <input
            type="checkbox"
            checked={cfg.idle_only}
            disabled={computeBusy}
            onchange={(e) =>
              setKey("compute.idle_only", String((e.currentTarget as HTMLInputElement).checked))}
          />
          <span class="row-text">
            <span class="row-title">Only scan while the host is idle</span>
            <span class="row-detail">Pauses the loop when CPU is busier than {Math.round(cfg.idle_cpu_threshold * 100)}%. Off = continuous.</span>
          </span>
        </label>
      </div>
    {/if}
  {:else}
    <V1Body muted text="Loading performance settings…" />
  {/if}
</div>

<!-- Startup -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-st-active);"></span><span>Startup</span></div>
  <label class="row">
    <input type="checkbox" bind:checked={launchAtLogin} />
    <span class="row-text">
      <span class="row-title">Launch Vaner at login</span>
      <span class="row-detail">Drops an XDG autostart entry under <code>~/.config/autostart/</code> on toggle. Daemon-side wiring lands in v0.2.3.</span>
    </span>
  </label>

  {#if $engineService}
    {@const svc = $engineService}
    {@const checked = svc.state === "active" || svc.state === "enabled"}
    {@const disabled = serviceBusy || svc.state === "unavailable"}
    {@const serviceInstalled = svc.state !== "missing" && svc.state !== "unavailable"}
    <label class="row" class:dim={disabled}>
      <input
        type="checkbox"
        {checked}
        {disabled}
        onchange={(e) => onServiceToggleClick((e.currentTarget as HTMLInputElement).checked)}
      />
      <span class="row-text">
        <span class="row-title">Run engine in the background (systemd)</span>
        <span class="row-detail">{describeServiceState(svc.state)}</span>
        {#if svc.workspace && serviceInstalled}
          <span class="row-detail">Targeting <code>{svc.workspace}</code> · unit at <code>{svc.unit_path}</code>.</span>
        {/if}
      </span>
    </label>

    {#if serviceInstalled}
      <label class="row" class:dim={lingerBusy}>
        <input
          type="checkbox"
          checked={svc.linger_enabled}
          disabled={lingerBusy}
          onchange={(e) => onLingerToggleClick((e.currentTarget as HTMLInputElement).checked)}
        />
        <span class="row-text">
          <span class="row-title">Keep the engine running after logout (linger)</span>
          <span class="row-detail">
            {#if svc.linger_enabled}
              The user manager survives logout, so Vaner keeps indexing across reboots and lock screens.
            {:else}
              The engine stops as soon as you log out and restarts on next login. Toggle on if you want it indexing in the background even when you're away.
            {/if}
          </span>
          <span class="row-detail">
            Toggling triggers a graphical password prompt (polkit) to run <code>loginctl {svc.linger_enabled ? "disable-linger" : "enable-linger"}</code>.
          </span>
        </span>
      </label>
    {/if}
  {/if}
</div>

<!-- Memory -->
<div class="card">
  <div class="card-head"><span class="rail" style="background: var(--vd-st-attention);"></span><span>Memory</span></div>
  <V1Body
    muted
    text="Vaner stores feedback, preferences, and learned signals locally."
  />
  <div class="actions">
    <V1GhostButton title="Export memory" onclick={exportMemory} />
    <V1GhostButton title="Open privacy view" onclick={() => showToast("Opens vaner.ai/privacy in v0.2.3.", "info", 3000)} />
    <V1GhostButton title="Clear memory…" destructive onclick={() => (confirmClear = true)} />
  </div>
  {#if confirmClear}
    <div class="confirm">
      <p>This deletes everything Vaner has learned about your work. Sources stay connected.</p>
      <div class="actions">
        <V1GhostButton title="Cancel" onclick={() => (confirmClear = false)} />
        <V1GhostButton title="Clear" destructive onclick={clearMemory} />
      </div>
    </div>
  {/if}
</div>

<style>
  .hd {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-bottom: 24px;
  }
  .card {
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    padding: 18px 20px;
    margin-bottom: 14px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .card-head {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
    margin-bottom: 4px;
  }
  .card-head .rail {
    width: 2px;
    height: 14px;
    border-radius: 1px;
    flex: 0 0 auto;
  }
  .hint {
    margin: 0;
    font-size: 11.5px;
    color: var(--vd-fg-3);
    line-height: 1.45;
  }
  .muted {
    margin: 0;
    font-size: 12px;
    color: var(--vd-fg-2);
    line-height: 1.5;
  }
  .bundle-row {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .bundle-name {
    font-size: 14px;
    font-weight: 500;
    color: var(--vd-fg-1);
    font-family: var(--vd-font-mono);
  }
  .tier {
    font-size: 10.5px;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    color: var(--vd-fg-3);
  }
  .actions {
    display: flex;
    gap: 6px;
    align-items: center;
    flex-wrap: wrap;
  }
  .row {
    display: flex;
    gap: 10px;
    align-items: flex-start;
    cursor: pointer;
  }
  .row.toggle-only {
    align-items: center;
  }
  .row.dim,
  .window.dim {
    opacity: 0.45;
    pointer-events: none;
  }
  .row input[type="checkbox"] {
    margin-top: 3px;
    accent-color: var(--vd-purple);
  }
  .row-text {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }
  .row-title {
    font-size: 13px;
    font-weight: 500;
    color: var(--vd-fg-1);
  }
  .row-detail {
    font-size: 11px;
    color: var(--vd-fg-3);
    line-height: 1.5;
  }
  .row-detail code {
    font-family: var(--vd-font-mono);
    font-size: 11px;
    color: var(--vd-fg-2);
  }
  .window {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 6px 4px;
    background: rgba(255, 255, 255, 0.02);
    border: 0.5px solid var(--vd-hair);
    border-radius: var(--vd-r-chip);
  }
  .window-label {
    font-size: 11px;
    color: var(--vd-fg-3);
    padding: 0 6px;
  }
  .window-time {
    font-family: var(--vd-font-mono);
    font-size: 12px;
    color: var(--vd-fg-1);
    min-width: 44px;
    text-align: center;
    font-variant-numeric: tabular-nums;
  }
  .step {
    width: 22px;
    height: 22px;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    color: var(--vd-fg-2);
    border: 0.5px solid var(--vd-line);
    border-radius: 4px;
    cursor: pointer;
    font-family: var(--vd-font);
    font-size: 14px;
    line-height: 1;
    padding: 0;
    transition: background 0.12s, color 0.12s;
  }
  .step:hover:not(:disabled) {
    background: var(--vd-bg-2);
    color: var(--vd-fg-1);
  }
  .step:disabled {
    cursor: not-allowed;
  }
  .confirm {
    margin-top: 6px;
    padding: 12px 14px;
    background: color-mix(in srgb, var(--vd-st-attention) 6%, transparent);
    border: 0.5px solid color-mix(in srgb, var(--vd-st-attention) 30%, transparent);
    border-radius: var(--vd-r-chip);
  }
  .confirm p {
    margin: 0 0 8px;
    font-size: 12px;
    color: var(--vd-fg-2);
    line-height: 1.5;
  }
  .workspace-row {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }
  .workspace-path {
    flex: 1 1 auto;
    min-width: 0;
    font-family: var(--vd-font-mono);
    font-size: 12px;
    color: var(--vd-fg-1);
    background: rgba(255, 255, 255, 0.04);
    border: 0.5px solid var(--vd-hair);
    border-radius: 6px;
    padding: 6px 10px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    user-select: text;
    -webkit-user-select: text;
  }
  .workspace-empty {
    flex: 1 1 auto;
    font-size: 12px;
    color: var(--vd-fg-3);
    font-style: italic;
  }
  .presets {
    display: flex;
    flex-direction: column;
    gap: 6px;
    margin-top: 4px;
  }
  .preset-row {
    display: grid;
    grid-template-columns: max-content 1fr max-content;
    grid-template-rows: auto auto;
    column-gap: 12px;
    row-gap: 2px;
    align-items: baseline;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-chip);
    text-align: left;
    cursor: pointer;
    color: var(--vd-fg-1);
    font: inherit;
    transition: background 120ms ease, border-color 120ms ease;
  }
  .preset-row:hover:not(:disabled) {
    background: var(--vd-bg-2);
  }
  .preset-row:disabled { opacity: 0.5; cursor: not-allowed; }
  .preset-row.selected {
    border-color: color-mix(in srgb, var(--vd-st-active) 60%, transparent);
    background: color-mix(in srgb, var(--vd-st-active) 8%, var(--vd-bg-1));
  }
  .preset-title {
    grid-column: 1;
    font-size: 13px;
    font-weight: 500;
  }
  .preset-sub {
    grid-column: 1 / span 3;
    grid-row: 2;
    font-size: 11.5px;
    color: var(--vd-fg-3);
    line-height: 1.4;
  }
  .preset-badge {
    grid-column: 3;
    grid-row: 1;
    font-size: 9.5px;
    font-weight: 600;
    letter-spacing: 0.1em;
    color: var(--vd-st-active);
    align-self: center;
  }
  .adv-toggle {
    align-self: flex-start;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    background: transparent;
    border: none;
    padding: 4px 0;
    color: var(--vd-fg-3);
    font: inherit;
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    cursor: pointer;
  }
  .adv-toggle:hover { color: var(--vd-fg-2); }
  .adv-block {
    display: flex;
    flex-direction: column;
    gap: 14px;
    padding-top: 4px;
  }
  .adv-explainer {
    margin: 0;
    font-size: 11.5px;
    color: var(--vd-fg-3);
    line-height: 1.5;
  }
  .adv-explainer em {
    color: var(--vd-fg-1);
    font-style: normal;
    font-weight: 500;
  }
  .slider-row {
    display: flex;
    flex-direction: column;
    gap: 6px;
    cursor: pointer;
  }
  .slider-label {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    gap: 12px;
    font-size: 12px;
    color: var(--vd-fg-1);
  }
  .slider-value {
    font-family: var(--vd-font-mono);
    font-size: 11.5px;
    color: var(--vd-fg-2);
  }
  .slider-row input[type="range"] {
    width: 100%;
    accent-color: var(--vd-st-active);
  }
  .slider-help {
    font-size: 11px;
    color: var(--vd-fg-3);
    line-height: 1.45;
  }
</style>
