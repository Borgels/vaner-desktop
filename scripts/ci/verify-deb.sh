#!/usr/bin/env bash
# verify-deb.sh — smoke-test the signatures produced by sign-deb.sh.
# Run immediately after signing in CI so a broken sig fails the job
# before the GitHub Release upload.

set -euo pipefail

deb_path="${1:?usage: verify-deb.sh <path-to-.deb>}"
deb_dir=$(dirname "$deb_path")

# Expect the repo's committed pubkey to match the private key we signed with.
pubkey_path="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/release-key.asc"
if [[ ! -f "$pubkey_path" ]]; then
  echo "ERROR: pubkey not committed at scripts/release-key.asc" >&2
  exit 2
fi

gnupghome=$(mktemp -d)
trap 'rm -rf "$gnupghome"' EXIT
chmod 700 "$gnupghome"
export GNUPGHOME="$gnupghome"

gpg --batch --import "$pubkey_path"

# Detached sig.
gpg --verify "$deb_path.asc" "$deb_path"

# SHA256SUMS sig.
gpg --verify "$deb_dir/SHA256SUMS.asc" "$deb_dir/SHA256SUMS"

# Cross-check each file's digest against SHA256SUMS.
pushd "$deb_dir" >/dev/null
sha256sum -c SHA256SUMS
popd >/dev/null

# Embedded dpkg-sig signature (optional — only if dpkg-sig is present).
if command -v dpkg-sig >/dev/null; then
  dpkg-sig --verify "$deb_path"
fi

echo "all signatures verified"
