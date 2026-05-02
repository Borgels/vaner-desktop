// VanerState + supporting types — ported 1:1 from
// vaner-desktop-macos/vaner/State/VanerState.swift and
// vaner-desktop-macos/vaner/Services/EngineClient.swift.
// The reducer at ./reducer.ts is the single producer of `VanerState`;
// every popover state component consumes a variant.

import type { PredictedPrompt, PreparedWorkCard } from "$lib/contract/types.js";

// -----------------------------------------------------------------------------
// Source / kinds
// -----------------------------------------------------------------------------

export type SourceKind =
  | "github"
  | "files"
  | "linear"
  | "slack"
  | "calendar"
  | "drive"
  | "mail"
  | "notion";

export interface SourceRef {
  id: string;
  kind: SourceKind;
  label: string;
  weight: number;
}

export type SourceStatusValue = "connected" | "paused" | "blocked" | "idle";

export interface SourceStatus {
  source: SourceRef;
  status: SourceStatusValue;
  detail: string;
}

// -----------------------------------------------------------------------------
// Engine status
// -----------------------------------------------------------------------------

export interface ReadingItem {
  source: SourceKind;
  title: string;
  since: string;
}

export type IndexingState =
  | { kind: "idle" }
  | { kind: "learning"; currentlyReading: ReadingItem[]; etaMinutes: number | null }
  | { kind: "reindexing"; progress: number };

export interface EngineStatus {
  reachable: boolean;
  /** True when the `vaner` CLI itself isn't installed on this machine.
   *  The reducer prefers `.notInstalled` over `.error` in that case so
   *  the UI gives the user an install link instead of a misleading
   *  "Engine error" panel. */
  cliMissing: boolean;
  filesWatched: number;
  sourcesCount: number;
  uptimeMinutes: number;
  lastCycleSecondsAgo: number | null;
  cycleIntervalSeconds: number;
  indexing: IndexingState;
}

// -----------------------------------------------------------------------------
// Prepared moments
// -----------------------------------------------------------------------------

export type PreparedStrength = "lead" | "supporting";

export interface PreparedMoment {
  id: string;
  title: string;
  prediction: string;
  why: string[];
  primarySource: SourceRef;
  sources: SourceRef[];
  confidence: number;
  strength: PreparedStrength;
  /** ms since epoch — easier to compare in TS than a Date instance. */
  readyAt: number;
  pinned: boolean;
}

export interface PreparedList {
  lead: PreparedMoment | null;
  supporting: PreparedMoment[];
  pendingWhenNoAgent: number;
}

// -----------------------------------------------------------------------------
// Conflicts / errors / agents / install flow
// -----------------------------------------------------------------------------

export interface ConflictEvidence {
  sideALabel: string;
  sideASnippet: string;
  sideBLabel: string;
  sideBSnippet: string;
}

export interface ConflictSummary {
  id: string;
  headline: string;
  sources: SourceRef[];
  evidence: ConflictEvidence | null;
}

export interface AgentSuggestion {
  id: string;
  displayName: string;
  bundleIdentifier: string | null;
}

export interface EngineError {
  message: string;
  port: number | null;
  incidentID: string | null;
}

export type InstallFlowState =
  | { kind: "notDetected" }
  | { kind: "checkingForEngine" }
  | { kind: "installing"; progress: number; logTail: string[] }
  | { kind: "upgradeAvailable"; current: string; latest: string }
  | { kind: "installed"; version: string }
  | { kind: "failed"; message: string };

// -----------------------------------------------------------------------------
// Aggregate state payloads (per VanerState variant)
// -----------------------------------------------------------------------------

export interface LearningProgress {
  filesWatched: number;
  uptimeMinutes: number;
  currentlyReading: ReadingItem[];
  etaMinutes: number | null;
}

export interface WatchingSummary {
  filesWatched: number;
  sourcesCount: number;
  preparedCount: number;
  currentlyReading: ReadingItem[];
  lastPreparedAgo: string | null;
}

// -----------------------------------------------------------------------------
// VanerState — discriminated union driven by the reducer
// -----------------------------------------------------------------------------

export type VanerState =
  | { kind: "needsWorkspace" }
  | { kind: "engineMissing"; install: InstallFlowState }
  | { kind: "notInstalled" }
  | { kind: "installedNotConnected" }
  | { kind: "learning"; progress: LearningProgress }
  | { kind: "watching"; summary: WatchingSummary; silentHours: boolean }
  | { kind: "prepared"; lead: PreparedMoment; supporting: PreparedMoment[] }
  | { kind: "preparedWork"; cards: PreparedWorkCard[] }
  | { kind: "attention"; conflict: ConflictSummary }
  | { kind: "permissionNeeded"; sources: SourceStatus[] }
  | { kind: "noActiveAgent"; pendingCount: number; suggestedLaunch: AgentSuggestion[] }
  | { kind: "activePredictions"; predictions: PredictedPrompt[] }
  | { kind: "error"; engine: EngineError }
  | { kind: "paused"; queued: number }
  | { kind: "idle" };

export type VanerStateKind = VanerState["kind"];
