<script lang="ts">
  import { toasts } from "$lib/stores/toast.js";
</script>

<div class="stack" aria-live="polite" aria-atomic="true">
  {#each $toasts as toast (toast.id)}
    <div class="toast tone-{toast.tone}">{toast.message}</div>
  {/each}
</div>

<style>
  .stack {
    position: fixed;
    left: 50%;
    bottom: 14px;
    transform: translateX(-50%);
    display: flex;
    flex-direction: column;
    gap: 6px;
    pointer-events: none;
    z-index: 100;
  }

  .toast {
    font-size: 12px;
    font-weight: 500;
    letter-spacing: -0.1px;
    padding: 8px 12px;
    border-radius: 7px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    color: var(--vd-fg-1);
    box-shadow: 0 4px 18px rgba(0, 0, 0, 0.35);
    pointer-events: auto;
  }

  .toast.tone-success {
    border-color: rgba(116, 194, 156, 0.35);
    color: var(--vd-st-on);
  }

  .toast.tone-attention {
    border-color: rgba(239, 106, 90, 0.4);
    color: var(--vd-st-attention);
  }
</style>
