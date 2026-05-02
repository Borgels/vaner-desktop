<!--
  NotWiredToAnyClient — fresh install, no MCP client has Vaner
  registered. Vaner is a first-class MCP server; without a client
  consuming it, surfacing engine state would be misleading. Steer
  the user to the integration docs and let them choose where to wire
  Vaner in (Cursor / Claude Code / Zed / …).

  Replaces the v0.2.3 .needsWorkspace popover, which mistakenly
  treated the desktop as a workspace-config tool. The desktop is a
  viewer + lifecycle helper for the daemon — workspace context is
  provided by whichever MCP client connects.
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

  // The reducer passes `detected` here for forward-compat (a future
  // wired-count display, etc.) but the popover is intentionally
  // copy-light — no scan stats, no helper text. Per-client detail
  // lives on docs.vaner.ai or in the Agents pane.
  type Props = {
    detected?: { total: number; wiredCount: number; wiredLabels: string[] };
  };
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  const _props: Props = $props();

  async function openIntegrationsDocs() {
    try {
      await invoke("open_external_url", {
        url: "https://docs.vaner.ai/integrations/connect-your-client",
      });
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : "Could not open the integrations docs.",
        "attention",
        4000,
      );
    }
  }

  async function openInAgents() {
    // Open the companion's Agents pane — that's where Vaner is
    // installed into the user's MCP clients (Cursor / Claude Code /
    // …). The +layout listener auto-rescans when the popover
    // regains focus, so the user closing the companion drops them
    // back here with the wired-count refreshed automatically.
    try {
      await invoke("open_companion", { tab: "agents" });
    } catch (err) {
      showToast(
        err instanceof Error ? err.message : `Could not open companion: ${err}`,
        "attention",
        4000,
      );
    }
  }
</script>

<QuietShell markState="idle" stateLabel="Not wired" stateLabelTint="var(--vd-amber)">
  <V1Headline text="Connect Vaner to a client to start using it." />

  <div class="actions">
    <V1PrimaryButton
      title="Connect a client…"
      tint="var(--vd-amber)"
      onclick={openInAgents}
    />
    <V1GhostButton title="docs.vaner.ai" onclick={openIntegrationsDocs} />
  </div>
</QuietShell>

{#snippet footer()}
  <PopoverFooter health="idle" detailsDisabled />
{/snippet}

<style>
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
    flex-wrap: wrap;
  }
</style>
