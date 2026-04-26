// Silent-hours toggle — surfaced from the Companion → Preferences pane.
// Stub for now: off.

import { writable, type Writable } from "svelte/store";

export const silentHours: Writable<boolean> = writable(false);
