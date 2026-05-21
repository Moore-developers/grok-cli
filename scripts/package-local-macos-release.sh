#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage:
  scripts/package-local-macos-release.sh <tag> [--upload]

Packages the local macOS Apple Silicon release binary and, optionally,
uploads it to the matching GitHub Release.

Examples:
  scripts/package-local-macos-release.sh v0.1.0
  scripts/package-local-macos-release.sh v0.1.0 --upload

Notes:
  - This script is intentionally local-only and targets aarch64-apple-darwin.
  - It does not produce macOS Intel, Linux, or Windows artifacts.
  - Use --upload only after the GitHub Release exists.
USAGE
}

tag="${1:-}"
upload="${2:-}"

if [[ "$tag" == "-h" || "$tag" == "--help" ]]; then
  usage
  exit 0
fi

if [[ -z "$tag" || "$upload" != "" && "$upload" != "--upload" || $# -gt 2 ]]; then
  usage >&2
  exit 2
fi

repo_root="$(git rev-parse --show-toplevel 2>/dev/null || true)"
if [[ -z "$repo_root" ]]; then
  echo "error: this script must be run inside the grok-cli git repository." >&2
  exit 1
fi
cd "$repo_root"

tag_commit="$(git rev-list -n 1 "$tag" 2>/dev/null || true)"
if [[ -z "$tag_commit" ]]; then
  echo "error: tag '${tag}' was not found locally." >&2
  echo "fetch tags first, or create the release tag before packaging." >&2
  exit 1
fi

head_commit="$(git rev-parse HEAD)"
if [[ "$head_commit" != "$tag_commit" ]]; then
  echo "error: HEAD does not match ${tag}." >&2
  echo "checkout the tagged commit before packaging the release asset." >&2
  exit 1
fi

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "error: working tree has uncommitted changes." >&2
  echo "commit or stash changes before packaging a release asset." >&2
  exit 1
fi

host_os="$(uname -s)"
host_arch="$(uname -m)"

if [[ "$host_os" != "Darwin" || "$host_arch" != "arm64" ]]; then
  echo "error: this script only packages macOS Apple Silicon artifacts." >&2
  echo "detected: ${host_os}/${host_arch}" >&2
  exit 1
fi

target="aarch64-apple-darwin"
asset="grok-cli-macos-${target}.tar.gz"
dist_dir="dist/release-${tag}"
package_dir="${dist_dir}/package"

cargo build --release --locked

rm -rf "$dist_dir"
mkdir -p "$package_dir"
cp target/release/grok-cli "$package_dir/grok-cli"

tar -C "$package_dir" -czf "${dist_dir}/${asset}" grok-cli
hash="$(shasum -a 256 "${dist_dir}/${asset}" | awk '{print $1}')"
printf "%s  %s\n" "$hash" "$asset" > "${dist_dir}/${asset}.sha256"

echo "Created:"
echo "  ${dist_dir}/${asset}"
echo "  ${dist_dir}/${asset}.sha256"

if [[ "$upload" == "--upload" ]]; then
  if ! command -v gh >/dev/null 2>&1; then
    echo "error: GitHub CLI 'gh' is required for --upload." >&2
    echo "install and authenticate gh, then rerun this command:" >&2
    echo "  gh release upload ${tag} ${dist_dir}/${asset} ${dist_dir}/${asset}.sha256 --clobber" >&2
    exit 1
  fi

  gh release upload "$tag" "${dist_dir}/${asset}" "${dist_dir}/${asset}.sha256" --clobber
  echo "Uploaded macOS Apple Silicon release assets to ${tag}."
fi
