# vaner-linux

Linux desktop companion for [Vaner](https://github.com/Borgels/Vaner) —
the predictive-context engine. Menu-bar / tray app that watches the
daemon's active predictions over SSE and lets the user adopt a
prepared package into whichever AI agent (Claude Code, Cursor, Zed,
etc.) is running.

Tauri v2 + SvelteKit. Rust backend depends on the shared
[`vaner-contract`](https://github.com/Borgels/Vaner/tree/main/crates/vaner-contract)
crate from the Vaner monorepo; the SwiftUI macOS sibling
([vaner-desktop](https://github.com/Borgels/vaner-desktop-macos)) uses the
same conformance fixtures to stay in lock-step without sharing a
runtime.

Target: **Ubuntu 22.04+ / Debian 12+**, X11 or KDE Wayland. Stock
GNOME on Wayland needs `gnome-shell-extension-appindicator` for the
tray icon to appear — the app detects this at first launch and
surfaces install guidance.

## Status

- [x] L1: `vaner-contract` crate (upstream)
- [x] L2: conformance fixtures bridge (upstream)
- [x] L3: Swift conformance test consumption (TODO — schedule with Vaner tag)
- [ ] L4: Tauri app skeleton *(this repo — in progress)*
- [ ] L5: tray + popover + SSE wiring
- [ ] L6: `.deb` packaging + CI release workflow
- [ ] L7: end-to-end ship gate on a fresh Ubuntu 24.04 VM

## Build

Prereqs:
```bash
# Ubuntu 24.04 system deps for WebKitGTK-based Tauri:
sudo apt install -y libwebkit2gtk-4.1-dev libgtk-3-dev \
  libayatana-appindicator3-dev librsvg2-dev patchelf
# Rust toolchain (1.85+ for edition 2024):
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Node 20+ and pnpm:
corepack enable && corepack prepare pnpm@latest --activate
```

Dev:
```bash
pnpm install
pnpm tauri dev
```

Build (no packaging yet — L6 wires up `.deb`):
```bash
pnpm tauri build --no-bundle
```

## Architecture

See `/home/abo/.claude/plans/task-finish-phase-d-async-reef.md` in the
original planning session; the short version:

```
┌─────────────────────────────────────────────┐
│          vaner daemon (Python)              │
│     /predictions/active  •  /events/stream  │
│     /predictions/{id}/adopt  •  /status     │
└──────────────────────┬──────────────────────┘
                       │ HTTP / SSE (loopback)
                       │
          ┌────────────▼──────────────┐
          │  vaner-contract (Rust)    │
          │  • models + enums         │
          │  • HTTP client + SSE      │
          │  • reducer + handoff      │
          └────────────┬──────────────┘
                       │ (compiled in)
                       │
          ┌────────────▼──────────────┐
          │  Tauri v2 Rust backend    │
          │  commands + SSE task      │
          └────────────┬──────────────┘
                       │ invoke / emit
                       │
          ┌────────────▼──────────────┐
          │   SvelteKit (WebView)     │
          │   QuietShell popover UI   │
          └───────────────────────────┘
```

Design tokens (`src/lib/tokens.css`) are vendored from Vaner's
`ui/cockpit/src/styles/tokens.css` so the visual language stays 1:1
with the web cockpit and the SwiftUI macOS app.

## License

Apache-2.0, inherited from the Vaner project.
