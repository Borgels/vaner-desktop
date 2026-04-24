#!/usr/bin/env bash
# sign-deb.sh — sign the built .deb with Vaner's release key.
#
# Expects these env vars (injected by the release workflow from GitHub
# Secrets — see docs/RELEASE_KEY_SETUP.md):
#
#   VANER_RELEASE_GPG_PRIVKEY     — base64(armored private key)
#   VANER_RELEASE_GPG_PASSPHRASE  — passphrase for the key
#   VANER_RELEASE_GPG_FINGERPRINT — expected 40-char fingerprint (sanity check)
#
# Produces, next to the input .deb:
#
#   <name>.deb        — signed in-place via `dpkg-sig --sign builder`
#   <name>.deb.asc    — detached ASCII-armored signature (belt + braces)
#   SHA256SUMS        — sha256 digests of the above
#   SHA256SUMS.asc    — detached signature over SHA256SUMS
#
# The detached signatures are redundant with dpkg-sig's embedded one —
# but `dpkg-sig` requires installing `dpkg-sig` on the user's machine
# to verify, while `gpg --verify` is universally available. We publish
# both so install.sh can pick the friendliest path.

set -euo pipefail

: "${VANER_RELEASE_GPG_PRIVKEY:?set VANER_RELEASE_GPG_PRIVKEY}"
: "${VANER_RELEASE_GPG_PASSPHRASE:?set VANER_RELEASE_GPG_PASSPHRASE}"
: "${VANER_RELEASE_GPG_FINGERPRINT:?set VANER_RELEASE_GPG_FINGERPRINT}"

deb_path="${1:?usage: sign-deb.sh <path-to-.deb>}"
if [[ ! -f "$deb_path" ]]; then
  echo "ERROR: .deb not found at $deb_path" >&2
  exit 2
fi

deb_dir=$(dirname "$deb_path")
deb_name=$(basename "$deb_path")

# Isolated GNUPGHOME so we never touch the runner's default keyring.
gnupghome=$(mktemp -d)
trap 'rm -rf "$gnupghome"' EXIT
chmod 700 "$gnupghome"
export GNUPGHOME="$gnupghome"

# Import the key.
echo "$VANER_RELEASE_GPG_PRIVKEY" | base64 -d | gpg --batch --import

# Sanity: the imported key must match the fingerprint we baked into the repo.
imported_fpr=$(gpg --list-secret-keys --with-colons | awk -F: '/^fpr:/ {print $10; exit}')
expected_fpr=${VANER_RELEASE_GPG_FINGERPRINT//[[:space:]]/}
if [[ "$imported_fpr" != "$expected_fpr" ]]; then
  echo "ERROR: imported key fingerprint '$imported_fpr' does not match expected '$expected_fpr'" >&2
  exit 3
fi

# Ensure dpkg-sig is available (runner bootstrap installs it).
if ! command -v dpkg-sig >/dev/null; then
  echo "ERROR: dpkg-sig not installed; sudo apt install dpkg-sig" >&2
  exit 4
fi

# Embedded signature (so `dpkg-sig --verify` on a user machine passes).
dpkg-sig --gpg-options "--pinentry-mode loopback --passphrase $VANER_RELEASE_GPG_PASSPHRASE" \
         --sign builder \
         --sign-key "$imported_fpr" \
         "$deb_path"

# Detached signature (simpler verification path via plain `gpg --verify`).
gpg --batch --pinentry-mode loopback \
    --passphrase "$VANER_RELEASE_GPG_PASSPHRASE" \
    --local-user "$imported_fpr" \
    --armor --detach-sign --output "$deb_path.asc" "$deb_path"

# SHA256SUMS + signed checksums file.
pushd "$deb_dir" >/dev/null
sha256sum "$deb_name" "$deb_name.asc" > SHA256SUMS
gpg --batch --pinentry-mode loopback \
    --passphrase "$VANER_RELEASE_GPG_PASSPHRASE" \
    --local-user "$imported_fpr" \
    --armor --detach-sign --output SHA256SUMS.asc SHA256SUMS
popd >/dev/null

echo
echo "signed artifacts:"
ls -la "$deb_path" "$deb_path.asc" "$deb_dir/SHA256SUMS" "$deb_dir/SHA256SUMS.asc"
