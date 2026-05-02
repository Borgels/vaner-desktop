<!--
  OllamaMissing — surfaced when Vaner's default local backend (Ollama
  on 127.0.0.1:11434) isn't reachable. Vaner's local-first model loop
  hard-depends on it, so showing "engine unavailable" without naming
  the actual cause was the failure mode this state replaces.

  Two CTAs:
    1. Install Ollama — primary. Spawns a terminal and pipes
       `curl https://ollama.com/install.sh | sh`. The user sees the
       install + sudo prompt happen interactively. After spawn, the
       Rust task pokes the cache so the popover flips out of this
       state on the next probe.
    2. Open download page — fallback. xdg-opens
       https://ollama.com/download in the user's browser, for users
       who'd rather grab the .deb / .pkg manually.

  Detail line surfaces whatever the Rust probe captured ("Ollama isn't
  installed.", "Ollama isn't responding."). Same component handles both
  "missing" and "installed but stopped" — only the headline copy
  changes.
-->
<script lang="ts">
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { showToast } from "$lib/stores/toast.js";
  import { installOllama } from "$lib/stores/ollama-health.js";

  type Props = { installed: boolean; detail: string };
  const { installed, detail }: Props = $props();

  let installing = $state(false);

  async function runInstaller() {
    if (installing) return;
    installing = true;
    try {
      await installOllama();
      showToast(
        "Ollama installer launched in a terminal — follow the prompts.",
        "info",
        4500,
      );
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not launch installer: ${err}`,
        "attention",
        5000,
      );
    } finally {
      installing = false;
    }
  }

  async function openDownloads() {
    try {
      await invoke("open_external_url", { url: "https://ollama.com/download" });
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not open browser: ${err}`,
        "attention",
        4000,
      );
    }
  }

  const headline = $derived(
    installed ? "Ollama isn't running." : "Ollama is missing.",
  );
  const body = $derived(
    installed
      ? "Vaner's local model runs on Ollama. Start the Ollama service so the model loop can use it."
      : "Vaner's local backend uses Ollama. Install it once and Vaner takes care of the rest.",
  );
</script>

<QuietShell markState="attention" stateLabel="Ollama needed" stateLabelTint="var(--vd-amber)">
  <V1Headline text={headline} />
  <V1Body muted text={body} />

  {#if detail}
    <p class="detail">{detail}</p>
  {/if}

  <div class="actions">
    {#if !installed}
      <V1PrimaryButton
        title={installing ? "Launching installer…" : "Install Ollama"}
        tint="var(--vd-amber)"
        onclick={runInstaller}
      />
    {/if}
    <V1GhostButton title="Open ollama.com" onclick={openDownloads} />
  </div>

  {#snippet footer()}
    <PopoverFooter health="attention" healthLabel="Ollama needed for local models" />
  {/snippet}
</QuietShell>

<style>
  .detail {
    margin: 6px 0 0;
    font-size: 11.5px;
    color: var(--vd-fg-3);
    font-family: var(--vd-font-mono);
  }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
    flex-wrap: wrap;
  }
</style>
