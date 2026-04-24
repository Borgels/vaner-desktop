#!/usr/bin/env bash
# verify-artifacts.sh — belt-and-braces pre-upload check on every
# signature produced by sign-artifacts.sh.

set -euo pipefail

bundle_dir=${1:?usage: verify-artifacts.sh <bundle-dir>}
pubkey_path="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)/release-key.asc"
[[ -f "$pubkey_path" ]] || { echo "ERROR: scripts/release-key.asc missing" >&2; exit 2; }

gnupghome=$(mktemp -d); chmod 700 "$gnupghome"
trap 'rm -rf "$gnupghome"' EXIT
export GNUPGHOME="$gnupghome"

gpg --batch --import "$pubkey_path"

# Detached sig for every .deb and .AppImage.
mapfile -t asc_files < <(find "$bundle_dir" -maxdepth 3 -type f -name "*.asc" | sort)
for sig in "${asc_files[@]}"; do
  payload="${sig%.asc}"
  [[ -f "$payload" ]] || { echo "orphan signature without payload: $sig" >&2; exit 3; }
  echo "→ verify $(basename "$payload")"
  gpg --verify "$sig" "$payload"
done

# SHA256SUMS + cross-check digests.
[[ -f "$bundle_dir/SHA256SUMS" && -f "$bundle_dir/SHA256SUMS.asc" ]] \
  || { echo "SHA256SUMS(.asc) missing in $bundle_dir" >&2; exit 4; }
gpg --verify "$bundle_dir/SHA256SUMS.asc" "$bundle_dir/SHA256SUMS"

# Run the checksum check from each leaf directory so the relative
# paths in SHA256SUMS resolve.
while read -r _ file; do
  path=$(find "$bundle_dir" -maxdepth 3 -type f -name "$file" | head -1)
  [[ -n "$path" ]] || { echo "missing listed file: $file" >&2; exit 5; }
  echo "→ checksum $file"
  (cd "$(dirname "$path")" && sha256sum -c <<<"$(awk -v f="$file" '$2==f' "$bundle_dir/SHA256SUMS")")
done < "$bundle_dir/SHA256SUMS"

# Embedded dpkg-sig check (only for .deb, only if tool present).
if command -v dpkg-sig >/dev/null; then
  while read -r d; do
    echo "→ dpkg-sig --verify $(basename "$d")"
    dpkg-sig --verify "$d"
  done < <(find "$bundle_dir" -maxdepth 3 -type f -name "*.deb")
fi

echo "all signatures verified"
