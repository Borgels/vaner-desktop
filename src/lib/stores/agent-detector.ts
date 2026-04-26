// Running-agent observation — populated by the Rust `detect_agents` command in
// WS8 (scans /proc/*/comm for known agent process names: cursor, claude, code,
// zed, etc.). Stub for now: nothing running.

import { writable, type Writable } from "svelte/store";
import type { AgentSuggestion } from "$lib/state/types.js";

export interface AgentDetectorSnapshot {
  runningCount: number;
  /** Suggested launches when noActiveAgent fires. */
  suggestions: AgentSuggestion[];
}

const empty: AgentDetectorSnapshot = { runningCount: 0, suggestions: [] };

export const agentDetector: Writable<AgentDetectorSnapshot> = writable(empty);
export function setAgentDetector(s: AgentDetectorSnapshot): void {
  agentDetector.set(s);
}
