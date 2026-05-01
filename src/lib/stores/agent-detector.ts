// Agent detector — populated by polling the Rust `detect_agents`
// command. On Linux it scans /proc/*/comm for known agent process
// names. On other OSs it returns the suggestion list with
// running_count = 0 so the .noActiveAgent state still has something
// to render.

import { invoke } from "@tauri-apps/api/core";
import { writable, type Writable } from "svelte/store";
import type { AgentSuggestion } from "$lib/state/types.js";

export interface AgentDetectorSnapshot {
  runningCount: number;
  suggestions: AgentSuggestion[];
}

const empty: AgentDetectorSnapshot = { runningCount: 0, suggestions: [] };

export const agentDetector: Writable<AgentDetectorSnapshot> = writable(empty);

export function setAgentDetector(s: AgentDetectorSnapshot): void {
  agentDetector.set(s);
}

interface AgentDetectorOut {
  running_count: number;
  suggestions: { id: string; display_name: string; bundle_identifier: string | null }[];
}

let pollHandle: ReturnType<typeof setInterval> | null = null;

export function startAgentDetectorPolling(intervalMs = 8000): void {
  if (pollHandle != null) return;
  const tick = async () => {
    try {
      const out = await invoke<AgentDetectorOut>("detect_agents");
      agentDetector.set({
        runningCount: out.running_count,
        suggestions: out.suggestions.map((s) => ({
          id: s.id,
          displayName: s.display_name,
          bundleIdentifier: s.bundle_identifier,
        })),
      });
    } catch {
      // Detector failing isn't fatal — leave the previous snapshot.
    }
  };
  void tick();
  pollHandle = setInterval(tick, intervalMs);
}

export function stopAgentDetectorPolling(): void {
  if (pollHandle != null) {
    clearInterval(pollHandle);
    pollHandle = null;
  }
}
