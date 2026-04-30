import { writable, type Readable } from "svelte/store";
import { invoke } from "@tauri-apps/api/core";
import type { PreparedWorkCard } from "$lib/contract/types.js";

const { subscribe, set }: { subscribe: Readable<PreparedWorkCard[]>["subscribe"]; set: (v: PreparedWorkCard[]) => void } =
  writable<PreparedWorkCard[]>([]);

let started = false;
let timer: ReturnType<typeof setInterval> | null = null;

export async function refreshPreparedWork(): Promise<void> {
  try {
    const cards = await invoke<PreparedWorkCard[]>("prepared_work", { limit: 3 });
    set(cards);
  } catch (err) {
    console.warn("[vaner] prepared_work failed:", err);
    set([]);
  }
}

export function startPreparedWorkPolling(): void {
  if (started) return;
  started = true;
  void refreshPreparedWork();
  timer = setInterval(() => void refreshPreparedWork(), 5000);
}

export function stopPreparedWorkPolling(): void {
  if (timer) {
    clearInterval(timer);
    timer = null;
  }
  started = false;
}

export const preparedWork = { subscribe };
