<!--
  Companion center pane router. Picks the right pane based on ?tab=.
-->
<script lang="ts">
  import { page } from "$app/stores";
  import PreparedPane from "./panes/PreparedPane.svelte";
  import SourcesPane from "./panes/SourcesPane.svelte";
  import AgentsPane from "./panes/AgentsPane.svelte";
  import ModelsPane from "./panes/ModelsPane.svelte";
  import EnginePane from "./panes/EnginePane.svelte";
  import PreferencesPane from "./panes/PreferencesPane.svelte";
  import DiagnosticsPane from "./panes/DiagnosticsPane.svelte";

  const active = $derived(($page.url.searchParams.get("tab") ?? "prepared").toLowerCase());
</script>

{#if active === "prepared"}
  <PreparedPane />
{:else if active === "sources"}
  <SourcesPane />
{:else if active === "agents"}
  <AgentsPane />
{:else if active === "models"}
  <ModelsPane />
{:else if active === "engine"}
  <EnginePane />
{:else if active === "preferences"}
  <PreferencesPane />
{:else if active === "diagnostics"}
  <DiagnosticsPane />
{:else}
  <PreparedPane />
{/if}
