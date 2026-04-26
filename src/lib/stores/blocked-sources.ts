// Blocked sources — populated by the Rust `blocked_sources` command in WS8.
// Stub for now: empty.

import { writable, type Writable } from "svelte/store";
import type { SourceStatus } from "$lib/state/types.js";

export const blockedSources: Writable<SourceStatus[]> = writable([]);
export function setBlockedSources(s: SourceStatus[]): void {
  blockedSources.set(s);
}
