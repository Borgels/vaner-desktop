<script lang="ts">
  import { needsAppIndicator } from "$lib/stores/app-state.js";

  function dismiss() {
    needsAppIndicator.set(false);
  }
</script>

{#if $needsAppIndicator}
  <div class="backdrop" role="dialog" aria-modal="true" aria-labelledby="firstrun-title">
    <div class="modal">
      <h2 id="firstrun-title">One more step</h2>
      <p>
        Vaner lives in your top bar, but GNOME on Wayland needs one
        extension to render tray icons.
      </p>
      <pre><code>sudo apt install gnome-shell-extension-appindicator</code></pre>
      <p class="hint">
        After it installs, log out and back in. KDE and X11 users
        don't need this step — they'll never see this message.
      </p>
      <button class="dismiss" onclick={dismiss}>Got it</button>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.55);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 200;
  }

  .modal {
    max-width: 360px;
    padding: 22px 20px 18px;
    background: var(--vd-bg-1);
    border: 0.5px solid var(--vd-line);
    border-radius: var(--vd-r-card);
    box-shadow: var(--vd-shadow-pop);
  }

  h2 {
    font-size: var(--vd-t-title);
    font-weight: 600;
    margin: 0 0 10px;
    color: var(--vd-fg-1);
  }

  p {
    font-size: var(--vd-t-body);
    color: var(--vd-fg-2);
    margin: 0 0 10px;
    line-height: 1.45;
  }

  p.hint {
    font-size: var(--vd-t-label);
    color: var(--vd-fg-3);
    margin-top: 12px;
  }

  pre {
    background: rgba(0, 0, 0, 0.25);
    padding: 10px 12px;
    border-radius: 6px;
    margin: 0 0 10px;
    overflow-x: auto;
  }

  code {
    font-family: var(--vd-font-mono);
    font-size: 12px;
    color: var(--vd-st-active);
  }

  .dismiss {
    font-family: var(--vd-font);
    font-size: 12px;
    font-weight: 500;
    padding: 7px 14px;
    margin-top: 6px;
    border: none;
    border-radius: 6px;
    background: var(--vd-fg-1);
    color: var(--vd-bg-0);
    cursor: pointer;
  }
</style>
