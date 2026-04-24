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

## Install

Signed `.deb` verified against Vaner's release key. Pipe-to-shell is
fine for this one because the script pins the fingerprint in its own
source — if the uploaded pubkey ever gets swapped, the install
aborts with a fingerprint-mismatch error.

```bash
curl -fsSL https://raw.githubusercontent.com/Borgels/vaner-desktop-linux/main/scripts/install.sh | bash
```

Or, for the paranoid (recommended) — read the script first:

```bash
curl -fsSL https://raw.githubusercontent.com/Borgels/vaner-desktop-linux/main/scripts/install.sh \
  -o install.sh
less install.sh     # confirm the VANER_RELEASE_FPR pin against docs/RELEASE_KEY_SETUP.md
bash install.sh
```

Manual verification path:

```bash
# Grab the latest release + detached signature
VER=$(curl -fsSL https://api.github.com/repos/Borgels/vaner-desktop-linux/releases/latest | jq -r .tag_name)
curl -LO https://github.com/Borgels/vaner-desktop-linux/releases/download/$VER/vaner_${VER#v}_amd64.deb
curl -LO https://github.com/Borgels/vaner-desktop-linux/releases/download/$VER/vaner_${VER#v}_amd64.deb.asc
curl -LO https://github.com/Borgels/vaner-desktop-linux/releases/download/$VER/release-key.asc

# Import + verify
gpg --import release-key.asc
gpg --verify vaner_${VER#v}_amd64.deb.asc vaner_${VER#v}_amd64.deb
sudo apt install ./vaner_${VER#v}_amd64.deb
```

The release pubkey fingerprint is pinned in [`scripts/install.sh`](scripts/install.sh);
cross-check against [docs/RELEASE_KEY_SETUP.md](docs/RELEASE_KEY_SETUP.md)
and `keys.openpgp.org`.

## Status

- [x] L1: `vaner-contract` crate (upstream)
- [x] L2: conformance fixtures bridge (upstream)
- [x] L3: Swift conformance test consumption (scheduled with Vaner tag)
- [x] L4: Tauri app skeleton
- [x] L5: tray + popover + menu + first-run AppIndicator modal
- [x] L6: signed `.deb` release workflow + install.sh verification
- [x] L7: Docker ship-gate + manual smoke runbook (see [`docs/SHIP_GATE.md`](docs/SHIP_GATE.md))

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
