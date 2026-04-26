// Reducer tests — port of vaner-desktop-macos/vanerTests/StateReducerTests.swift.
// Covers every branch of the precedence chain in reducer.ts. Each test
// builds a minimal `ReducerInputs` and asserts the resulting kind.

import { describe, expect, it } from "vitest";
import { reduce, type ReducerInputs } from "./reducer.js";
import type {
  AgentSuggestion,
  EngineStatus,
  PreparedList,
  PreparedMoment,
  SourceRef,
  SourceStatus,
} from "./types.js";
import type { PredictedPrompt } from "$lib/contract/types.js";

// ---------- helpers ----------

const reachableStatus = (
  override: Partial<EngineStatus> = {},
): EngineStatus => ({
  reachable: true,
  filesWatched: 12,
  sourcesCount: 3,
  uptimeMinutes: 14,
  lastCycleSecondsAgo: 30,
  cycleIntervalSeconds: 60,
  indexing: { kind: "idle" },
  ...override,
});

const emptyPrepared = (): PreparedList => ({
  lead: null,
  supporting: [],
  pendingWhenNoAgent: 0,
});

const githubSrc: SourceRef = {
  id: "github",
  kind: "github",
  label: "vaner-desktop",
  weight: 1,
};

const baseInputs = (override: Partial<ReducerInputs> = {}): ReducerInputs => ({
  status: reachableStatus(),
  prepared: emptyPrepared(),
  blockedSources: [],
  anyAgentRunning: true,
  silentHours: false,
  hasAnySource: true,
  activePredictions: [],
  noAgentSuggestions: [],
  ...override,
});

const aMoment = (id: string, conf = 0.7): PreparedMoment => ({
  id,
  title: `Moment ${id}`,
  prediction: "do the thing",
  why: ["a", "b"],
  primarySource: githubSrc,
  sources: [githubSrc],
  confidence: conf,
  strength: "lead",
  readyAt: 0,
  pinned: false,
});

const aPrediction = (
  readiness: "ready" | "drafting" | "queued",
  confidence = 0.7,
): PredictedPrompt =>
  ({
    id: `${readiness}-${confidence}`,
    spec: {
      label: `Prediction ${readiness}`,
      hypothesis_type: "explicit_intent",
      source: "engine",
      confidence,
      specificity: "exact",
    },
    run: { readiness, started_at: "2026-04-26T20:00:00Z" },
    artifacts: {},
  }) as unknown as PredictedPrompt;

const blockedSrc = (): SourceStatus => ({
  source: githubSrc,
  status: "blocked",
  detail: "Token expired",
});

const fakeAgent = (id: string): AgentSuggestion => ({
  id,
  displayName: id,
  bundleIdentifier: null,
});

// ---------- tests ----------

describe("StateReducer precedence chain", () => {
  it("engine unreachable → .error", () => {
    const out = reduce(baseInputs({ status: reachableStatus({ reachable: false }) }));
    expect(out.kind).toBe("error");
  });

  it("blocked sources → .permissionNeeded (even if otherwise prepared)", () => {
    const out = reduce(
      baseInputs({
        blockedSources: [blockedSrc()],
        prepared: { ...emptyPrepared(), lead: aMoment("a") },
      }),
    );
    expect(out.kind).toBe("permissionNeeded");
  });

  it("no sources configured → .installedNotConnected", () => {
    const out = reduce(baseInputs({ hasAnySource: false }));
    expect(out.kind).toBe("installedNotConnected");
  });

  it("indexing learning → .learning", () => {
    const out = reduce(
      baseInputs({
        status: reachableStatus({
          indexing: { kind: "learning", currentlyReading: [], etaMinutes: 10 },
        }),
      }),
    );
    expect(out.kind).toBe("learning");
  });

  it("ready prediction + agent running → .activePredictions", () => {
    const out = reduce(
      baseInputs({
        activePredictions: [aPrediction("ready", 0.8)],
      }),
    );
    expect(out.kind).toBe("activePredictions");
  });

  it("ready prediction + no agent → .noActiveAgent", () => {
    const out = reduce(
      baseInputs({
        anyAgentRunning: false,
        activePredictions: [aPrediction("ready"), aPrediction("drafting")],
        noAgentSuggestions: [fakeAgent("Cursor")],
      }),
    );
    expect(out.kind).toBe("noActiveAgent");
    if (out.kind === "noActiveAgent") {
      expect(out.pendingCount).toBe(2);
    }
  });

  it("queued predictions are filtered out (not surfacable)", () => {
    const out = reduce(
      baseInputs({
        activePredictions: [aPrediction("queued")],
      }),
    );
    expect(out.kind).toBe("watching"); // falls through
  });

  it("activePredictions sorts ready before drafting, then by confidence desc", () => {
    const out = reduce(
      baseInputs({
        activePredictions: [
          aPrediction("drafting", 0.9),
          aPrediction("ready", 0.6),
          aPrediction("ready", 0.8),
          aPrediction("drafting", 0.7),
        ],
      }),
    );
    if (out.kind !== "activePredictions") throw new Error("expected activePredictions");
    const ord = out.predictions.map((p) => `${p.run.readiness}-${p.spec.confidence}`);
    expect(ord).toEqual(["ready-0.8", "ready-0.6", "drafting-0.9", "drafting-0.7"]);
  });

  it("prepared moment + agent → .prepared", () => {
    const out = reduce(
      baseInputs({
        prepared: { lead: aMoment("a"), supporting: [aMoment("b")], pendingWhenNoAgent: 0 },
      }),
    );
    expect(out.kind).toBe("prepared");
    if (out.kind === "prepared") {
      expect(out.lead.id).toBe("a");
      expect(out.supporting).toHaveLength(1);
    }
  });

  it("prepared moment + no agent → .noActiveAgent (pendingCount = lead + supporting)", () => {
    const out = reduce(
      baseInputs({
        anyAgentRunning: false,
        prepared: {
          lead: aMoment("a"),
          supporting: [aMoment("b"), aMoment("c")],
          pendingWhenNoAgent: 0,
        },
        noAgentSuggestions: [fakeAgent("Claude Desktop")],
      }),
    );
    expect(out.kind).toBe("noActiveAgent");
    if (out.kind === "noActiveAgent") {
      expect(out.pendingCount).toBe(3);
    }
  });

  it("no signals → .watching", () => {
    const out = reduce(baseInputs());
    expect(out.kind).toBe("watching");
    if (out.kind === "watching") {
      expect(out.silentHours).toBe(false);
    }
  });

  it("no signals + silent hours → .watching with flag", () => {
    const out = reduce(baseInputs({ silentHours: true }));
    expect(out.kind).toBe("watching");
    if (out.kind === "watching") {
      expect(out.silentHours).toBe(true);
    }
  });
});
