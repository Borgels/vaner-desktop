#!/usr/bin/env bash
#
# Vaner desktop installer.
#
# Downloads the latest signed .deb from the vaner-desktop-linux GitHub
# Release, verifies it against Vaner's release pubkey, and installs.
#
#   curl -fsSL https://raw.githubusercontent.com/Borgels/vaner-desktop-linux/main/scripts/install.sh | bash
#
# Or, for a specific version:
#
#   VANER_DESKTOP_VERSION=v0.1.0 \
#     curl -fsSL https://raw.githubusercontent.com/Borgels/vaner-desktop-linux/main/scripts/install.sh | bash
#
# Refuses to install if the signature doesn't verify or the fingerprint
# doesn't match the pin below. That pin is the only verification
# anchor — double-check against docs/RELEASE_KEY_SETUP.md + the repo's
# README if you're the paranoid kind (you should be).

set -euo pipefail

# --- PINNED FINGERPRINT ---------------------------------------------
# Once the real key lands on main, this line gets rewritten to the
# 40-character fingerprint from RELEASE_KEY_SETUP.md step 2. Until
# then the install script refuses to run — by design.
VANER_RELEASE_FPR="REPLACE_WITH_40_CHAR_FINGERPRINT_AT_KEY_LAUNCH"
# --------------------------------------------------------------------

REPO="Borgels/vaner-desktop-linux"
VERSION="${VANER_DESKTOP_VERSION:-latest}"
PUBKEY_URL="https://raw.githubusercontent.com/${REPO}/main/scripts/release-key.asc"

die() { echo "vaner-install: $*" >&2; exit 1; }

[[ "$VANER_RELEASE_FPR" =~ ^[A-F0-9]{40}$ ]] \
  || die "fingerprint pin not set — this install script hasn't been wired to the release key yet."

command -v curl   >/dev/null || die "curl is required"
command -v gpg    >/dev/null || die "gpg is required (sudo apt install gnupg)"
command -v dpkg   >/dev/null || die "this installer is Debian/Ubuntu-only"

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
curl -fsSL -o "$work/release-key.asc" "$PUBKEY_URL"

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
