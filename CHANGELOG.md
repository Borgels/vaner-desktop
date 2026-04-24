# Changelog

## [Unreleased]

## [0.1.0] - 2026-04-24

Initial release of the Vaner Linux desktop companion.

### Added (L5 — UX)
- System tray with colored Vaner brand mark (sourced from the
  `docs/handoff/vaner-desktop/brand/` package). Both left-click and
  right-click surface the menu (Linux convention).
- Tray menu: **Open Vaner** / Preferences… / Pause / Quit. Open Vaner
  pops the borderless popover window; the other items fire Tauri
  events the Svelte layer consumes.
- Popover lifecycle (`src-tauri/src/popover.rs`): show, hide, toggle,
  and auto-hide on `Focused(false)`. Position via
  `tauri-plugin-positioner::Position::TrayCenter` on X11; fallback
  on Wayland compositors that refuse fine-grained positioning.
- First-run AppIndicator-missing modal (`FirstRunGuidance.svelte`)
  triggered by `setup:appindicator-missing` from the Rust session
  probe. One-time, dismissable, copy-paste install command.
- Toast store + stack (`$lib/stores/toast.ts`,
  `$lib/components/ToastStack.svelte`). Adopt success/error, menu
  events, and future UI feedback route through it.

### Added (L6 — Release)
- **Dedicated Vaner release GPG key** signs every `.deb`. Onboarding
  runbook at [`docs/RELEASE_KEY_SETUP.md`](docs/RELEASE_KEY_SETUP.md)
  covers generation, GitHub Secrets upload, keyserver publication,
  and rotation policy.
- `scripts/ci/sign-deb.sh` — embedded (`dpkg-sig`) + detached
  (`.deb.asc`) + signed `SHA256SUMS.asc`. Fingerprint sanity-check
  against the repo-committed pubkey before any signing happens.
- `scripts/ci/verify-deb.sh` — runs immediately after sign, red CI
  if any signature fails verification.
- `.github/workflows/release.yml` — fires on `v*.*.*` tags, builds
  with `tauri-action@v0`, signs, verifies, publishes the GitHub
  Release. Dry-run path via `workflow_dispatch` so we can exercise
  the pipeline without cutting tags. Hard-fails if the repo still
  holds the placeholder fingerprint or placeholder pubkey.
- `scripts/install.sh` — user-facing bootstrap. Downloads the
  signed `.deb` + detached sig from the GitHub Release, imports the
  committed pubkey, **pins** the fingerprint in its own source
  (install aborts on mismatch), `apt install`s.
- `scripts/release-key.asc` — placeholder; swap for the real
  armored export before tagging v0.1.0. Refusal logic in
  release.yml enforces this.

### Added (L7 — Ship gate)
- `Dockerfile.ship-gate` — reproducible Ubuntu 24.04 environment
  matching what end users run.
- `scripts/ship-gate.sh` — end-to-end automated test: signature
  verify, `apt install`, daemon boot, /predictions/active fetch,
  Adopt POST, handoff-file write, `/vaner:next` step-0 simulation,
  `apt purge` cleanup check.
- [`docs/SHIP_GATE.md`](docs/SHIP_GATE.md) — runbook for both the
  automated Docker gate and the manual-smoke checklist before tagging.

### Changed
- Repo renamed from `vaner-linux` to `vaner-desktop-linux` to align
  with `vaner-desktop-macos`. GitHub redirects the old URL; internal
  references updated to the canonical name.

### Added (scope extension — pre-v0.1.0, not v0.1.1)
- **`.AppImage` bundle target.** Both `.deb` and `.AppImage` built,
  GPG-signed (detached `.asc`) and listed in `SHA256SUMS.asc` on every
  release.
- **Tauri auto-updater (`tauri-plugin-updater`).** Background check
  on app start emits `update:available` to the Svelte layer, which
  renders a calm top banner (`UpdateBanner.svelte`) with Install/
  Later actions and an inline progress bar. Updates verified against
  a dedicated minisign key (separate trust domain from the GPG `.deb`
  signing key) whose pubkey is embedded in `tauri.conf.json`.
- **Signed apt repository via GitHub Pages.**
  `scripts/ci/build-apt-repo.sh` runs `reprepro` against the release
  GPG key to produce a Debian dists/pool structure; the release
  workflow pushes the result to the `gh-pages` branch for GitHub
  Pages to serve. Users install via a standard
  `deb [signed-by=…] https://borgels.github.io/vaner-desktop-linux stable main`
  line; `apt upgrade` pulls new releases automatically. Landing
  page index.html + `.nojekyll` so Pages serves dotfile directories
  verbatim.
- `scripts/install.sh` default mode switched to `apt` (registers the
  repo + `apt install`); `VANER_MODE=deb` keeps the one-off .deb
  flow for ephemeral installs and CI.
- `docs/RELEASE_KEY_SETUP.md` grew Part 2 (minisign updater key) +
  Part 3 (GitHub Pages apt-repo setup + optional `apt.vaner.ai`
  custom domain via CNAME).

### Deferred to 0.1.1
- Preferences window content.
- Signed updater delta diffs (full-bundle replacement for now).

### Changed
- Apt repository is published at `https://apt.vaner.ai` (custom
  domain on the gh-pages branch via `CNAME`). Install URLs in
  `scripts/install.sh`, README, RELEASE_KEY_SETUP, and the Vaner
  docs site all point at the new host. The fallback
  `https://borgels.github.io/vaner-desktop-linux` stays valid via
  the GitHub-managed rewrite.
- `pnpm-lock.yaml` committed. CI + release workflows both run
  `pnpm install --frozen-lockfile` now — deterministic installs
  across runs, and lock drift fails loudly in CI instead of
  silently reconciling.



### Added
- Initial scaffold: Tauri v2 + SvelteKit + shared `vaner-contract`
  Rust crate (pinned to `feat/vaner-contract-crate` on the Vaner
  monorepo until L1 lands on main).
- SvelteKit popover layout with readiness pills, confidence meter,
  and Adopt button (calm-primary-only; disabled when row isn't
  adoptable).
- Tauri commands:
  - `active_predictions()` → `Vec<PredictedPrompt>`
  - `adopt_prediction(predictionId)` → `String` intent
- Background SSE task (`sse_task`) bridging
  `vaner_contract::stream_prediction_events` to the WebView via
  `app.emit("predictions:snapshot", ...)`.
- Adopt-handoff flow:
  1. POST `/predictions/{id}/adopt` via the shared HTTP client.
  2. Drop the raw Resolution at `$XDG_STATE_HOME/vaner/pending-adopt.json`
     (handled off the Tauri async runtime via `spawn_blocking`).
  3. Paste-fallback clipboard: `predicted_response ?? prepared_briefing
     ?? intent`.
- First-run guidance scaffold: detects Wayland + GNOME without the
  AppIndicator extension and emits `setup:appindicator-missing` so
  the UI can nudge the user (UI modal itself TODO in L5).
- CI: `pnpm check` / `pnpm build` / `cargo fmt` / `cargo clippy` /
  `cargo check` on `ubuntu-22.04`.

### Deferred to L5
- Tray-icon lifecycle (click to toggle popover).
- Popover window positioning (`tauri-plugin-positioner`, tray-center
  on X11, top-right fallback on Wayland).
- First-run AppIndicator-missing modal UI.
- Theme toggle (light/dark via `.vd-light` class).

### Deferred to L6
- `.deb` bundle signing and release workflow.
- AppImage follow-up (v1.1).

### Known limitations (v0.1 scaffold)
- No tray icon yet; the app shows its main window directly.
- Icons in `src-tauri/icons/` are intentionally absent — add before
  release builds. `bundle.active = false` keeps CI happy.
- `@tauri-apps/plugin-clipboard-manager` / `plugin-dialog` listed as
  dependencies but the UI only uses clipboard (via the Rust side) in
  the adopt flow; dialog is there for L5's settings sheet.
