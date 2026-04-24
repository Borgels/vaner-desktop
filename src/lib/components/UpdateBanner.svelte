<script lang="ts">
  import { availableUpdate, updateProgress, installUpdate } from "$lib/stores/updater.js";

  function dismiss() {
    availableUpdate.set(null);
  }
</script>

{#if $availableUpdate}
  <div class="banner" role="status" aria-live="polite">
    <div class="copy">
      <span class="label">Update available</span>
      <span class="version">
        {$availableUpdate.currentVersion} → <strong>{$availableUpdate.version}</strong>
      </span>
    </div>
    <div class="actions">
      {#if $updateProgress === null}
        <button class="install" onclick={installUpdate}>Install</button>
        <button class="later" onclick={dismiss}>Later</button>
      {:else}
        <div class="progress" aria-label="Update progress">
          <div class="bar" style="width: {Math.round(($updateProgress ?? 0) * 100)}%"></div>
        </div>
      {/if}
    </div>
  </div>
{/if}

<style>
  .banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 8px 14px;
    background: var(--vd-bg-1);
    border-bottom: 0.5px solid var(--vd-line);
  }

  .copy {
    display: flex;
    flex-direction: column;
    gap: 1px;
  }

  .label {
    font-size: 10px;
    font-weight: 600;
    letter-spacing: 1px;
    text-transform: uppercase;
    color: var(--vd-st-active);
  }

  .version {
    font-size: 12px;
    color: var(--vd-fg-2);
    font-family: var(--vd-font-mono);
  }

  .actions {
    display: flex;
    gap: 6px;
    align-items: center;
  }

  button {
    font-family: var(--vd-font);
    font-size: 11px;
    padding: 5px 10px;
    border-radius: 5px;
    border: 0.5px solid var(--vd-line);
    background: transparent;
    color: var(--vd-fg-2);
    cursor: pointer;
  }

  button.install {
    background: var(--vd-st-active);
    color: var(--vd-bg-0);
    border-color: transparent;
    font-weight: 500;
  }

  .progress {
    width: 120px;
    height: 4px;
    border-radius: 2px;
    background: var(--vd-hair);
    overflow: hidden;
  }

  .bar {
    height: 100%;
    background: var(--vd-st-active);
    transition: width 0.18s ease;
  }
</style>
