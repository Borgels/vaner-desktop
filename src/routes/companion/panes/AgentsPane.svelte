<!--
  Agents pane — wraps the existing MCPClientsPanel from /preferences,
  re-skinned at its container level. Underlying logic (clients store +
  install/uninstall/doctor invokes) is unchanged.
-->
<script lang="ts">
  import V1Kicker from "$lib/components/primitives/V1Kicker.svelte";
  import V1Headline from "$lib/components/primitives/V1Headline.svelte";
  import V1Body from "$lib/components/primitives/V1Body.svelte";
  import MCPClientsPanel from "../../preferences/MCPClientsPanel.svelte";
</script>

<header class="hd">
  <V1Kicker text="Agents" />
  <V1Headline text="Who can Vaner talk to" size={22} />
  <V1Body
    muted
    text="MCP clients (Cursor, Claude Desktop, Zed, VS Code, …) discover Vaner via a tiny config file under each app's settings. Install once and the agent picks up your prepared moments automatically."
  />
</header>

<section class="panel">
  <MCPClientsPanel />
</section>

<style>
  .hd { display: flex; flex-direction: column; gap: 6px; margin-bottom: 24px; }
  .panel {
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    background: var(--vd-bg-1);
    padding: 16px;
  }
</style>
