#!/usr/bin/env bash
# sign-artifacts.sh — sign every .deb, .AppImage, and NSIS .exe in a
# bundle directory with Vaner's release GPG key. Produces detached
# .asc sidecars and a single signed SHA256SUMS file for the whole set.
#
# The embedded `dpkg-sig` signature is applied to .deb files only.
# Other formats (.AppImage, Windows .exe) use detached-signature-only.
# The Tauri-produced minisign `.AppImage.sig` / `.exe.sig` files are
# left untouched — they're a separate trust chain used by
# tauri-plugin-updater.
#
# Required env (from GitHub Secrets):
#   VANER_RELEASE_GPG_PRIVKEY     — base64(armored private key)
#   VANER_RELEASE_GPG_PASSPHRASE  — key passphrase
#   VANER_RELEASE_GPG_FINGERPRINT — expected 40-char fingerprint
#
# Usage:
#   sign-artifacts.sh <dir-containing-built-bundles>
#
# The dir is typically src-tauri/target/release/bundle/ and contains
# deb/<name>.deb and appimage/<name>.AppImage. The script recurses
# once to pick up both.

set -euo pipefail

: "${VANER_RELEASE_GPG_PRIVKEY:?set VANER_RELEASE_GPG_PRIVKEY}"
: "${VANER_RELEASE_GPG_PASSPHRASE:?set VANER_RELEASE_GPG_PASSPHRASE}"
: "${VANER_RELEASE_GPG_FINGERPRINT:?set VANER_RELEASE_GPG_FINGERPRINT}"

bundle_dir=${1:?usage: sign-artifacts.sh <bundle-dir>}
[[ -d "$bundle_dir" ]] || { echo "ERROR: not a directory: $bundle_dir" >&2; exit 2; }

# Work in an isolated GNUPGHOME so nothing touches the runner's keyring.
gnupghome=$(mktemp -d); chmod 700 "$gnupghome"
trap 'rm -rf "$gnupghome"' EXIT
export GNUPGHOME="$gnupghome"

echo "$VANER_RELEASE_GPG_PRIVKEY" | base64 -d | gpg --batch --import

imported_fpr=$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr:/ {print $10; exit}')
expected_fpr=${VANER_RELEASE_GPG_FINGERPRINT//[[:space:]]/}
if [[ "$imported_fpr" != "$expected_fpr" ]]; then
  echo "ERROR: imported key fingerprint '$imported_fpr' does not match expected '$expected_fpr'" >&2
  exit 3
fi

gpg_sign() {
  # $1 = file to sign → produces $1.asc next to it
  gpg --batch --pinentry-mode loopback \
      --passphrase "$VANER_RELEASE_GPG_PASSPHRASE" \
      --local-user "$imported_fpr" \
      --armor --detach-sign --output "$1.asc" "$1"
}

# Collect artifacts. tauri-bundler puts .deb under deb/ and .AppImage
# under appimage/. We don't assume those exact names — just glob.
mapfile -t debs      < <(find "$bundle_dir" -maxdepth 3 -type f -name "*.deb"      | sort)
mapfile -t appimages < <(find "$bundle_dir" -maxdepth 3 -type f -name "*.AppImage" | sort)
mapfile -t exes      < <(find "$bundle_dir" -maxdepth 3 -type f -name "*-setup.exe" | sort)

if ((${#debs[@]} + ${#appimages[@]} + ${#exes[@]} == 0)); then
  echo "ERROR: no .deb, .AppImage, or .exe bundles found under $bundle_dir" >&2
  exit 4
fi

# Sign each artifact.
if ((${#debs[@]})); then
  command -v dpkg-sig >/dev/null || { echo "dpkg-sig missing; apt install dpkg-sig" >&2; exit 5; }
  for d in "${debs[@]}"; do
    echo "→ signing .deb (embedded + detached): $d"
    dpkg-sig -k "$imported_fpr" \
             --gpg-options "--pinentry-mode loopback --passphrase $VANER_RELEASE_GPG_PASSPHRASE" \
             --sign builder \
             "$d"
    gpg_sign "$d"
  done
fi

for a in "${appimages[@]}"; do
  echo "→ signing .AppImage (detached): $a"
  gpg_sign "$a"
done

for e in "${exes[@]}"; do
  echo "→ signing Windows .exe (detached): $e"
  gpg_sign "$e"
done

# Single consolidated SHA256SUMS at the bundle-dir root, signed.
sums_path="$bundle_dir/SHA256SUMS"
{
  for d in "${debs[@]}";      do (cd "$(dirname "$d")" && sha256sum "$(basename "$d")" "$(basename "$d").asc"); done
  for a in "${appimages[@]}"; do (cd "$(dirname "$a")" && sha256sum "$(basename "$a")" "$(basename "$a").asc"); done
  for e in "${exes[@]}";      do (cd "$(dirname "$e")" && sha256sum "$(basename "$e")" "$(basename "$e").asc"); done
} > "$sums_path"

gpg --batch --pinentry-mode loopback \
    --passphrase "$VANER_RELEASE_GPG_PASSPHRASE" \
    --local-user "$imported_fpr" \
    --armor --detach-sign --output "$sums_path.asc" "$sums_path"

echo
echo "signed artifacts:"
for d in "${debs[@]}";      do ls -la "$d" "$d.asc"; done
for a in "${appimages[@]}"; do ls -la "$a" "$a.asc"; done
for e in "${exes[@]}";      do ls -la "$e" "$e.asc"; done
ls -la "$sums_path" "$sums_path.asc"
