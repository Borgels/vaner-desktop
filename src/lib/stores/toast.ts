import { writable } from "svelte/store";

export type ToastTone = "success" | "info" | "attention";

export interface Toast {
  id: number;
  message: string;
  tone: ToastTone;
  durationMs: number;
}

const { subscribe, update } = writable<Toast[]>([]);
let nextId = 1;

export function showToast(message: string, tone: ToastTone = "info", durationMs = 3000): void {
  const id = nextId++;
  update((list) => [...list, { id, message, tone, durationMs }]);
  if (durationMs > 0) {
    setTimeout(() => {
      update((list) => list.filter((t) => t.id !== id));
    }, durationMs);
  }
}

export const toasts = { subscribe };
