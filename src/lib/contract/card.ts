// 0.8.5 — Card-model display helpers.
//
// The daemon (`vaner.intent.readiness`) is the source of truth for the
// readiness + ETA labels. When it populates `eta_bucket_label` /
// `readiness_label` directly on a `PredictedPrompt`, prefer those — they
// guarantee glyph parity (en-dash, NBSP, etc.) across every client.
// These helpers are the fallback path for pre-0.8.5 daemons that emit
// only the raw enum values, plus a stable lookup the UI can use to
// localise labels later without round-tripping the daemon.

import type { EtaBucket, PredictedPrompt, Readiness } from "./types";

/** En-dash typographically matches the daemon source — do NOT replace
 *  with a hyphen-minus. The Rust conformance fixtures + Python tests
 *  pin this exact glyph; any drift fails CI. */
export const ETA_BUCKET_LABELS: Record<EtaBucket, string> = {
  ready_now: "Ready now",
  under_20s: "~10–20s",
  under_1m: "~1 min",
  working: "Working",
  maturing: "Maturing in background",
};

export const READINESS_LABELS: Record<Readiness, string> = {
  queued: "Queued",
  grounding: "Grounding",
  evidence_gathering: "Gathering evidence",
  drafting: "Drafting",
  ready: "Ready",
  stale: "Stale",
};

/** Resolve a card's ETA bucket label, preferring the server-supplied
 *  string and falling back to the canonical enum→label map. */
export function etaBucketLabel(card: PredictedPrompt): string | null {
  if (typeof card.eta_bucket_label === "string" && card.eta_bucket_label) {
    return card.eta_bucket_label;
  }
  if (card.eta_bucket && card.eta_bucket in ETA_BUCKET_LABELS) {
    return ETA_BUCKET_LABELS[card.eta_bucket];
  }
  return null;
}

/** Resolve a card's readiness label, preferring server-supplied. */
export function readinessLabel(card: PredictedPrompt): string {
  if (typeof card.readiness_label === "string" && card.readiness_label) {
    return card.readiness_label;
  }
  return READINESS_LABELS[card.run.readiness] ?? card.run.readiness;
}

/** True when the card is click-Adopt-eligible. Server-supplied
 *  `adoptable` wins; otherwise fall back to the raw readiness gate
 *  (pre-0.8.5 daemons). */
export function cardIsAdoptable(card: PredictedPrompt): boolean {
  if (typeof card.adoptable === "boolean") {
    return card.adoptable;
  }
  return card.run.readiness === "ready" || card.run.readiness === "drafting";
}
