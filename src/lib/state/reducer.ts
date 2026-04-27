// Pure reducer — ported line-by-line from
// vaner-desktop-macos/vaner/State/StateReducer.swift (lines 39–114).
// Same precedence chain, same fallbacks, same input shape. Tests at
// ./reducer.test.ts cover every branch with the macOS fixtures.
//
// Do NOT optimize. Do NOT short-circuit. The chain is the contract;
// any deviation between Linux and macOS shows up as a different popover
// state for the same daemon condition, which is exactly the bug a single
// pure reducer is supposed to prevent.

import { isAdoptable, type PredictedPrompt } from "$lib/contract/types.js";
import type {
  AgentSuggestion,
  EngineStatus,
  LearningProgress,
  PreparedList,
  SourceStatus,
  VanerState,
  WatchingSummary,
} from "./types.js";

export interface ReducerInputs {
  status: EngineStatus;
  prepared: PreparedList;
  blockedSources: SourceStatus[];
  anyAgentRunning: boolean;
  silentHours: boolean;
  hasAnySource: boolean;
  /** 0.8.0 prediction-centric pondering. Defaults to [] for callers
   *  that haven't been updated to the new shape. */
  activePredictions: PredictedPrompt[];
  /** Suggested agents to launch when noActiveAgent fires. Equivalent
   *  to the macOS `PreviewData.noAgentSuggestions` constant; injected
   *  here so the reducer stays pure (no static-data import). */
  noAgentSuggestions: AgentSuggestion[];
  /** Tray-menu Pause toggle. When true, the popover renders a calm
   *  .paused state with a Resume button. Urgent states (error,
   *  permissionNeeded, attention, engineMissing) still show through
   *  so the user isn't silenced into a broken engine. */
  paused: boolean;
}

export function reduce(i: ReducerInputs): VanerState {
  // 1. Engine unreachable → .error (overrides pause; the user needs
  //    to know the engine is down even if they asked for quiet)
  if (!i.status.reachable) {
    return {
      kind: "error",
      engine: {
        message: "The Vaner engine isn't responding on localhost.",
        port: null,
        incidentID: null,
      },
    };
  }

  // 2. Any blocked sources (expired auth) → .permissionNeeded
  //    (also overrides pause — auth needs the user)
  if (i.blockedSources.length > 0) {
    return { kind: "permissionNeeded", sources: i.blockedSources };
  }

  // 3. No sources configured → .installedNotConnected
  //    (overrides pause — the user can't have meant to pause an
  //    engine that hasn't started yet)
  if (!i.hasAnySource) {
    return { kind: "installedNotConnected" };
  }

  // 4. Paused: count anything in flight so the user knows what
  //    Vaner is holding back, then short-circuit to .paused.
  //    Comes after the urgent-3 states so an error during pause
  //    still surfaces.
  if (i.paused) {
    const queued =
      i.activePredictions.filter((p) => isAdoptable(p.run.readiness)).length +
      (i.prepared.lead ? 1 : 0) +
      i.prepared.supporting.length;
    return { kind: "paused", queued };
  }

  // 4. Currently learning → .learning
  if (i.status.indexing.kind === "learning") {
    const progress: LearningProgress = {
      filesWatched: i.status.filesWatched,
      uptimeMinutes: i.status.uptimeMinutes,
      currentlyReading: i.status.indexing.currentlyReading,
      etaMinutes: i.status.indexing.etaMinutes,
    };
    return { kind: "learning", progress };
  }

  // 5. 0.8.0 — predictions in drafting/ready outrank a reactive
  //    .prepared moment. Symmetric with .prepared: if no agent is
  //    running, redirect to .noActiveAgent so the user launches one
  //    before adopting (the Resolution would land in a pending-adopt
  //    file no one is watching otherwise).
  const surfacable = i.activePredictions.filter((p) => isAdoptable(p.run.readiness));
  if (surfacable.length > 0) {
    if (!i.anyAgentRunning) {
      return {
        kind: "noActiveAgent",
        pendingCount: surfacable.length,
        suggestedLaunch: i.noAgentSuggestions,
      };
    }
    const sorted = [...surfacable].sort((lhs, rhs) => {
      if (lhs.run.readiness !== rhs.run.readiness) {
        // .ready before .drafting
        return lhs.run.readiness === "ready" ? -1 : 1;
      }
      return rhs.spec.confidence - lhs.spec.confidence;
    });
    return { kind: "activePredictions", predictions: sorted };
  }

  // 6. Reactive prepared moment(s) exist → .prepared (or .noActiveAgent
  //    when nothing's running to receive the handoff).
  if (i.prepared.lead) {
    if (!i.anyAgentRunning) {
      return {
        kind: "noActiveAgent",
        pendingCount: 1 + i.prepared.supporting.length,
        suggestedLaunch: i.noAgentSuggestions,
      };
    }
    return {
      kind: "prepared",
      lead: i.prepared.lead,
      supporting: i.prepared.supporting,
    };
  }

  // 7. Default → .watching (alive and reading, nothing strong yet).
  const summary: WatchingSummary = {
    filesWatched: i.status.filesWatched,
    sourcesCount: i.status.sourcesCount,
    preparedCount: 0,
    currentlyReading: [],
    lastPreparedAgo: null,
  };
  return { kind: "watching", summary, silentHours: i.silentHours };
}
