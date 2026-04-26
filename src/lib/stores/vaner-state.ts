// vaner-state.ts — the popover's single source of truth. A Svelte `derived`
// store that runs the pure reducer (src/lib/state/reducer.ts) over the
// observable inputs and produces a `VanerState` discriminated union. Every
// popover state component subscribes to this; the route at
// src/routes/+page.svelte switches on `kind`.

import { derived, type Readable } from "svelte/store";
import { reduce, type ReducerInputs } from "$lib/state/reducer.js";
import type { VanerState } from "$lib/state/types.js";
import { agentDetector } from "./agent-detector.js";
import { blockedSources } from "./blocked-sources.js";
import { engineStatus } from "./engine-status.js";
import { predictions } from "./predictions.js";
import { prepared } from "./prepared.js";
import { silentHours } from "./silent-hours.js";

export const vanerState: Readable<VanerState> = derived(
  [predictions, engineStatus, prepared, blockedSources, agentDetector, silentHours],
  ([$preds, $status, $prep, $blocked, $agents, $silent]) => {
    const hasAnySource = $status.sourcesCount > 0;
    const inputs: ReducerInputs = {
      status: $status,
      prepared: $prep,
      blockedSources: $blocked,
      anyAgentRunning: $agents.runningCount > 0,
      silentHours: $silent,
      hasAnySource,
      activePredictions: $preds,
      noAgentSuggestions: $agents.suggestions,
    };
    return reduce(inputs);
  },
);
