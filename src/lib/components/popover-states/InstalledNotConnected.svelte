<!--
  InstalledNotConnected — Vaner engine is up but no MCP client (Cursor,
  Claude Desktop, Zed, VS Code, …) is wired to call it yet. v0.2.3
  reframe: Vaner is an MCP server, not a data-source client. The card
  used to ask the user to "connect a source" — that was misleading. The
  real first-run integration step is: install Vaner's MCP entry into
  the user's agent.
-->
<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import QuietShell from "$lib/components/primitives/QuietShell.svelte";
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import V1PrimaryButton from "$lib/components/primitives/V1PrimaryButton.svelte";
  import V1GhostButton from "$lib/components/primitives/V1GhostButton.svelte";
  import PopoverFooter from "$lib/components/PopoverFooter.svelte";

  // Quick-link list of the MCP clients the desktop app already knows
  // how to install into. Glyphs/labels match agent_detector.rs.
  const agents = [
    { kind: "claude-code", label: "Claude Code", detail: "CLI + IDE plugin" },
    { kind: "cursor", label: "Cursor", detail: "MCP entry under Settings" },
    { kind: "zed", label: "Zed", detail: "Settings → Context servers" },
    { kind: "vscode", label: "VS Code", detail: "Workspace MCP config" },
  ] as const;

  function openAgentsPane() {
    invoke("open_companion", { tab: "agents" }).catch(() => {});
  }
</script>

<QuietShell markState="idle" stateLabel="Wire up an agent">
  <V1Kicker text="Engine is running" />
  <div class="gap-6"></div>
  <V1Headline text="Wire up an agent to use Vaner" />
  <div class="gap-8"></div>
  <V1Body
    muted
    text="Vaner is an MCP server — your coding agent calls it for prepared context. Install the MCP entry into the agent you use, and Vaner will start preparing work for it."
  />

  <ul class="agent-list">
    {#each agents as a (a.kind)}
      <li class="agent-row">
        <span class="agent-label">{a.label}</span>
        <span class="agent-detail">{a.detail}</span>
      </li>
    {/each}
  </ul>

  <div class="actions">
    <V1PrimaryButton title="Open Agents" onclick={openAgentsPane} />
  </div>

  {#snippet footer()}
    <PopoverFooter health="idle" healthLabel="Engine running, no agents wired" />
  {/snippet}
</QuietShell>

<style>
  .gap-6 { height: 6px; }
  .gap-8 { height: 8px; }
  .agent-list {
    list-style: none;
    margin: 14px 0 0;
    padding: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .agent-row {
    display: flex;
    justify-content: space-between;
    gap: 10px;
    padding: 10px 12px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    color: var(--vd-fg-1);
    font-family: var(--vd-font);
    font-size: 13px;
  }
  .agent-label { font-weight: 500; }
  .agent-detail { color: var(--vd-fg-3); font-size: 11.5px; }
  .actions {
    display: flex;
    gap: 6px;
    margin-top: 14px;
  }
</style>
