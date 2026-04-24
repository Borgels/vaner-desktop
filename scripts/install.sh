#!/usr/bin/env bash
#
# Vaner desktop installer.
#
# Default behaviour: adds the Vaner apt repository and installs
# `vaner-desktop-linux` via apt so subsequent `apt upgrade` runs pull
# new releases automatically. Falls back to a direct .deb download
# (with detached-signature verification) if apt / sudo isn't
# available or the user explicitly opts out.
#
#   # recommended
#   curl -fsSL https://raw.githubusercontent.com/Borgels/vaner-desktop-linux/main/scripts/install.sh | bash
#
#   # one-off install, no apt-repo registration:
#   VANER_MODE=deb curl -fsSL .../install.sh | bash
#
#   # pinned version:
#   VANER_DESKTOP_VERSION=v0.1.0 VANER_MODE=deb curl -fsSL .../install.sh | bash
#
# Regardless of mode, the script refuses to install unless the
# downloaded pubkey's fingerprint matches the pin below. That pin is
# the only trust anchor — double-check against RELEASE_KEY_SETUP.md
# and keys.openpgp.org if you're paranoid (you should be).

set -euo pipefail

# --- PINNED FINGERPRINT ---------------------------------------------
# 40-character fingerprint of the Vaner release GPG key. The install
# script verifies the downloaded pubkey's fingerprint against this
# value before trusting any signature — changing this line is a
# social event that deserves an announcement in the repo README.
#
# Source of truth: docs/RELEASE_KEY_SETUP.md
# Verify: `gpg --fingerprint release@vaner.ai` after importing
#         scripts/release-key.asc
VANER_RELEASE_FPR="506B8FA959917D530E5EE7203D219B47A7E4F046"
# --------------------------------------------------------------------

REPO="Borgels/vaner-desktop-linux"
VERSION="${VANER_DESKTOP_VERSION:-latest}"
MODE="${VANER_MODE:-apt}"  # apt (default) or deb (one-off)
APT_ORIGIN="https://borgels.github.io/vaner-desktop-linux"
PUBKEY_URL_DIRECT="https://raw.githubusercontent.com/${REPO}/main/scripts/release-key.asc"
PUBKEY_URL_APT="${APT_ORIGIN}/release-key.asc"

die() { echo "vaner-install: $*" >&2; exit 1; }

[[ "$VANER_RELEASE_FPR" =~ ^[A-F0-9]{40}$ ]] \
  || die "fingerprint pin not set — this install script hasn't been wired to the release key yet."

command -v curl   >/dev/null || die "curl is required"
command -v gpg    >/dev/null || die "gpg is required (sudo apt install gnupg)"
command -v dpkg   >/dev/null || die "this installer is Debian/Ubuntu-only"

# --- apt-repo path --------------------------------------------------
if [[ "$MODE" == "apt" ]]; then
  command -v sudo >/dev/null || die "MODE=apt needs sudo; rerun as root or VANER_MODE=deb"

  work=$(mktemp -d); trap 'rm -rf "$work"' EXIT

  echo "→ fetching Vaner release pubkey from the apt repo origin …"
  if ! curl -fsSL "$PUBKEY_URL_APT" -o "$work/release-key.asc"; then
    echo "   (apt origin didn't respond — falling back to the raw GitHub URL)"
    curl -fsSL "$PUBKEY_URL_DIRECT" -o "$work/release-key.asc" \
      || die "could not fetch the release pubkey"
  fi

  export GNUPGHOME="$work/gnupg"; mkdir -p "$GNUPGHOME"; chmod 700 "$GNUPGHOME"
  gpg --batch --import "$work/release-key.asc" >/dev/null 2>&1
  actual_fpr=$(gpg --list-keys --with-colons | awk -F: '/^fpr:/ {print $10; exit}')
  if [[ "$actual_fpr" != "$VANER_RELEASE_FPR" ]]; then
    die "pubkey fingerprint mismatch!
       expected: $VANER_RELEASE_FPR
       got:      $actual_fpr
     aborting. Grab a fresh install.sh and try again."
  fi

  echo "→ registering apt-signed keyring + source list …"
  sudo install -d -m 0755 /etc/apt/keyrings
  gpg --dearmor < "$work/release-key.asc" | sudo tee /etc/apt/keyrings/vaner.gpg >/dev/null
  echo "deb [signed-by=/etc/apt/keyrings/vaner.gpg] ${APT_ORIGIN} stable main" \
    | sudo tee /etc/apt/sources.list.d/vaner.list >/dev/null

  echo "→ apt update …"
  sudo apt update

  echo "→ apt install vaner-desktop-linux …"
  sudo apt install -y vaner-desktop-linux

  echo
  echo "Installed via apt. Future releases arrive through \`apt upgrade\`."
  exit 0
fi

# --- direct .deb path ------------------------------------------------
[[ "$MODE" == "deb" ]] || die "unknown VANER_MODE='$MODE' (want 'apt' or 'deb')"

# Resolve the release manifest.
if [[ "$VERSION" == "latest" ]]; then
  api_url="https://api.github.com/repos/${REPO}/releases/latest"
else
  api_url="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
fi

echo "→ resolving release metadata for ${VERSION} …"
release_json=$(curl -fsSL -H "Accept: application/vnd.github+json" "$api_url") \
  || die "could not fetch release metadata from $api_url"

deb_url=$(printf '%s' "$release_json" | grep -Eo '"browser_download_url":[[:space:]]*"[^"]*\.deb"' | head -1 | sed -E 's/.*"([^"]+)"/\1/')
sig_url=$(printf '%s' "$release_json" | grep -Eo '"browser_download_url":[[:space:]]*"[^"]*\.deb\.asc"' | head -1 | sed -E 's/.*"([^"]+)"/\1/')

[[ -n "$deb_url" ]] || die "no .deb in the ${VERSION} release"
[[ -n "$sig_url" ]] || die "no .deb.asc in the ${VERSION} release — refusing to install unsigned package"

work=$(mktemp -d); trap 'rm -rf "$work"' EXIT

echo "→ downloading .deb + detached signature …"
curl -fsSL -o "$work/vaner.deb"     "$deb_url"
curl -fsSL -o "$work/vaner.deb.asc" "$sig_url"
curl -fsSL -o "$work/release-key.asc" "$PUBKEY_URL_DIRECT"

# Isolated keyring — never touch the user's default GNUPGHOME.
export GNUPGHOME="$work/gnupg"
mkdir -p "$GNUPGHOME"
chmod 700 "$GNUPGHOME"

echo "→ importing pubkey and checking fingerprint …"
gpg --batch --import "$work/release-key.asc" >/dev/null 2>&1
actual_fpr=$(gpg --list-keys --with-colons | awk -F: '/^fpr:/ {print $10; exit}')

if [[ "$actual_fpr" != "$VANER_RELEASE_FPR" ]]; then
  die "pubkey fingerprint mismatch!
       expected: $VANER_RELEASE_FPR
       got:      $actual_fpr
     aborting install. Either the release-key.asc on main was tampered
     with, or this install.sh is older than the current key. Grab a
     fresh install.sh from the repo and try again."
fi

echo "→ verifying .deb signature …"
gpg --batch --verify "$work/vaner.deb.asc" "$work/vaner.deb" \
  || die "detached signature failed to verify — refusing to install."

echo "→ signature OK. installing via apt …"
sudo apt install -y "$work/vaner.deb"

echo
echo "Vaner desktop installed. Launch it from your app menu, or:"
echo "  vaner-desktop-linux   # once the binary is on your PATH"
echo
echo "The first-run popover on GNOME/Wayland may prompt you to install"
echo "gnome-shell-extension-appindicator — that's expected."
