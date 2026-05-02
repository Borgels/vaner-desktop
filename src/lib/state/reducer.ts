// Pure reducer — ported line-by-line from
// vaner-desktop-macos/vaner/State/StateReducer.swift (lines 39–114).
// Same precedence chain, same fallbacks, same input shape. Tests at
// ./reducer.test.ts cover every branch with the macOS fixtures.
//
// Do NOT optimize. Do NOT short-circuit. The chain is the contract;
// any deviation between Linux and macOS shows up as a different popover
// state for the same daemon condition, which is exactly the bug a single
// pure reducer is supposed to prevent.

import { isAdoptable, type PredictedPrompt, type PreparedWorkCard } from "$lib/contract/types.js";
import type {
  AgentSuggestion,
  ClientDetectStatus,
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
  /** Snapshot of which AI clients have Vaner registered. When zero
   *  clients are wired in, the desktop has no consumer for the
   *  daemon — surfacing engine state ahead of "go integrate Vaner
   *  somewhere" is putting the cart before the horse. The popover
   *  routes to `.notWiredToAnyClient` until the detector confirms
   *  ≥1 wired client. Defaults are tolerant: if `detected.total = 0`
   *  the detector hasn't run yet (or failed), and we don't gate on
   *  it — falling through to whichever engine state is real. */
  clientDetect: ClientDetectStatus;
  /** 0.8.0 prediction-centric pondering. Defaults to [] for callers
   *  that haven't been updated to the new shape. */
  activePredictions: PredictedPrompt[];
  preparedWork: PreparedWorkCard[];
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
  // 1a. Vaner CLI itself isn't installed → .notInstalled. This MUST
  //     come before the unreachable branch: a fresh `vaner-desktop`
  //     install on a machine that's never seen the CLI would
  //     otherwise show "Engine error / restart engine", which is
  //     misleading. .notInstalled gives a real install link.
  if (i.status.cliMissing) {
    return { kind: "notInstalled" };
  }

  // 1b. No MCP client has Vaner wired in → .notWiredToAnyClient.
  //     This MUST sit before the engine-reachability check: with
  //     production-mode auto-bring-up disabled, a fresh install
  //     legitimately has no daemon running until a client invokes
  //     `vaner mcp`. Showing .error in that window is wrong — the
  //     engine isn't broken, it's idle because no consumer exists.
  //     `total === 0` (detector hasn't completed yet) is treated
  //     as wiredCount === 0 too, since "show the wire-a-client
  //     panel for half a second on cold start" is a better UX than
  //     "flash a scary engine-error then settle into the right
  //     state."
  if (i.clientDetect.wiredCount === 0) {
    return { kind: "notWiredToAnyClient", detected: i.clientDetect };
  }

  // 1c. Clients are wired but the engine isn't reachable. Now it's
  //     a real error — the user expects Vaner to be live somewhere
  //     and the cockpit is silent. Overrides pause; this is
  //     actionable.
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
      i.preparedWork.length +
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

  if (i.preparedWork.length > 0) {
    return { kind: "preparedWork", cards: i.preparedWork };
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
