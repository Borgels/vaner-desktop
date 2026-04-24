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
}

export function isAdoptable(r: Readiness): boolean {
  return r === "ready" || r === "drafting";
}
