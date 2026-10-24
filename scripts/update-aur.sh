#!/usr/bin/env bash
# Sync PKGBUILD + .SRCINFO with the latest stfu GitHub release.
# Modeled after the production pipeline in synara-aur.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
UPSTREAM_REPO="${UPSTREAM_REPO:-fireflylabss/stfu}"
AUR_DIR="${ROOT}/packaging/aur"
cd "$AUR_DIR"

need() { command -v "$1" >/dev/null 2>&1 || { echo "$1 is required" >&2; exit 1; }; }
need gh
need jq
need python3

release_json="$(gh api "repos/${UPSTREAM_REPO}/releases/latest")"
tag="$(jq -r '.tag_name' <<<"$release_json")"
prerelease="$(jq -r '.prerelease' <<<"$release_json")"

if [[ "$prerelease" == "true" ]]; then
  echo "Latest release $tag is a prerelease; skipping AUR update."
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    echo "updated=false" >>"$GITHUB_OUTPUT"
  fi
  exit 0
fi

if [[ ! "$tag" =~ ^v?([0-9]+\.[0-9]+\.[0-9]+.*)$ ]]; then
  echo "Unsupported tag format: $tag (expected vX.Y.Z or X.Y.Z)" >&2
  exit 1
fi

version="${BASH_REMATCH[1]}"
current_ver="$(sed -n 's/^pkgver=//p' PKGBUILD | head -n1)"

if [[ "$current_ver" == "$version" && "${FORCE_UPDATE:-false}" != "true" ]]; then
  echo "Already at $version; nothing to do."
  if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
    echo "updated=false" >>"$GITHUB_OUTPUT"
  fi
  exit 0
fi

tarball_url="https://github.com/${UPSTREAM_REPO}/archive/refs/tags/v${version}.tar.gz"
echo "Fetching tarball to compute sha256: $tarball_url"
tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT
curl -fsSL "$tarball_url" -o "$tmp"
source_sha="$(sha256sum "$tmp" | awk '{print $1}')"

echo "Updating PKGBUILD: $current_ver -> $version (sha256: $source_sha)"

python3 - "$version" "$source_sha" <<'PY'
import pathlib, sys

version, source_sha = sys.argv[1:]
path = pathlib.Path("PKGBUILD")
text = path.read_text()
lines = text.splitlines(keepends=True)
out = []
i = 0
while i < len(lines):
    line = lines[i]
    if line.startswith("pkgver="):
        out.append(f"pkgver={version}\n")
    elif line.startswith("pkgrel="):
        out.append("pkgrel=1\n")
    elif line.startswith("sha256sums=("):
        out.append(line)
        i += 1
        while i < len(lines) and not lines[i].strip().startswith(")"):
            i += 1
        out.append(f"  '{source_sha}'\n")
        if i < len(lines):
            out.append(lines[i])
    else:
        out.append(line)
    i += 1
path.write_text("".join(out))
PY

if command -v makepkg >/dev/null 2>&1; then
  makepkg --printsrcinfo > .SRCINFO
else
  echo "makepkg unavailable; .SRCINFO must be regenerated in arch container." >&2
fi

if [[ -n "${GITHUB_OUTPUT:-}" ]]; then
  {
    echo "updated=true"
    echo "version=$version"
    echo "tag=$tag"
    echo "previous=$current_ver"
  } >>"$GITHUB_OUTPUT"
fi

echo "Updated PKGBUILD to $version"
