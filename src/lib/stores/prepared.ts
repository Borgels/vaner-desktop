// Prepared moments — populated by the Rust `prepared_list` command in WS8.
// Stub for now: empty.

import { writable, type Writable } from "svelte/store";
import type { PreparedList } from "$lib/state/types.js";

const empty: PreparedList = { lead: null, supporting: [], pendingWhenNoAgent: 0 };

export const prepared: Writable<PreparedList> = writable(empty);
export function setPrepared(p: PreparedList): void {
  prepared.set(p);
}
