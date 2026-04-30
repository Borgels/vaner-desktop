// TypeScript mirrors of the `vaner-contract` Rust types.
//
// For v0.1 these are hand-written to avoid a codegen step in the dev
// loop. A follow-up lands `cargo test --features ts-rs --package
// vaner-contract` output here and CI enforces the committed copy
// matches the Rust source. Until then, update these in lock-step with
// changes to `crates/vaner-contract/src/{models,enums}.rs`.

export type PredictionSource =
  | "arc"
  | "pattern"
  | "llm_branch"
  | "macro"
  | "history"
  | "goal";

export type HypothesisType = "likely_next" | "possible_branch" | "long_tail";
export type Specificity = "concrete" | "category" | "anchor";
export type Readiness =
  | "queued"
  | "grounding"
  | "evidence_gathering"
  | "drafting"
  | "ready"
  | "stale";

/** 0.8.5 WS6 — coarse ETA bucket for a prediction. */
export type EtaBucket =
  | "ready_now"
  | "under_20s"
  | "under_1m"
  | "working"
  | "maturing";

export interface PredictionSpec {
  label: string;
  description: string | null;
  source: PredictionSource;
  anchor: string | null;
  confidence: number;
  hypothesis_type: HypothesisType;
  specificity: Specificity;
  created_at: number;
}

export interface PredictionRun {
  weight: number;
  token_budget: number;
  tokens_used: number;
  model_calls: number;
  scenarios_spawned: number;
  scenarios_complete: number;
  readiness: Readiness;
  updated_at: number;
}

export interface PredictionArtifacts {
  scenario_ids: string[];
  evidence_score: number;
  has_draft: boolean;
  has_briefing: boolean;
  thinking_trace_count: number;
}

export interface PredictedPrompt {
  id: string;
  spec: PredictionSpec;
  run: PredictionRun;
  artifacts: PredictionArtifacts;

  // 0.8.5 WS9 — UI card-model derivations. Server-computed. Optional so
  // pre-0.8.5 daemon responses continue to decode unchanged.
  readiness_label?: string;
  eta_bucket?: EtaBucket;
  eta_bucket_label?: string;
  adoptable?: boolean;
  rank?: number;
  ui_summary?: string;
  suppression_reason?: string;
  source_label?: string;
}

export type PreparedWorkSourceType = "work_product" | "prediction";
export type PreparedWorkKind =
  | "review"
  | "bug"
  | "docs"
  | "diff"
  | "brief"
  | "draft"
  | "prediction";
export type PreparedWorkActionKind = "inspect" | "export" | "adopt" | "dismiss" | "feedback";

export interface PreparedWorkAction {
  kind: PreparedWorkActionKind;
  label: string;
  tool: string | null;
  endpoint: string | null;
  arguments: Record<string, unknown>;
}

export interface PreparedWorkCard {
  id: string;
  source_id: string;
  source_type: PreparedWorkSourceType;
  kind: PreparedWorkKind;
  title: string;
  summary: string;
  badge: string;
  confidence_label: string;
  freshness_label: string;
  freshness_state?: "fresh" | "recent" | "possibly_stale" | "stale" | string;
  target_label: string;
  why_prepared?: string;
  action_note?: string;
  evidence_count: number;
  created_at: number;
  updated_at: number;
  primary_action: PreparedWorkAction | null;
  secondary_actions: PreparedWorkAction[];
  diagnostic_refs?: Array<Record<string, string>>;
}

export function isAdoptable(r: Readiness): boolean {
  return r === "ready" || r === "drafting";
}
