#!/bin/bash
# Build backback from a clean master checkout and install it to /Applications.
# The installed Info.plist names the commit it was built from (BBCommit).
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
target="${BB_INSTALL_DIR:-/Applications}/backback.app"

branch="$(git -C "$repo_dir" rev-parse --abbrev-ref HEAD)"
if [ "$branch" != "master" ] && [ "${BB_INSTALL_ANY_BRANCH:-}" != "1" ]; then
  echo "refused: install builds from master, this checkout is on $branch" >&2
  exit 2
fi
if [ -n "$(git -C "$repo_dir" status --porcelain --untracked-files=no)" ]; then
  echo "refused: the checkout has uncommitted changes" >&2
  exit 2
fi
if pgrep -f "$target/Contents/MacOS/backback" >/dev/null; then
  echo "refused: $target is running; quit it first" >&2
  exit 2
fi

built="$(bash "$repo_dir/docs/release/scripts/bb_bundle.sh" | tail -1)"
rm -rf "${target:?}"
ditto "$built" "$target"
codesign --verify --deep --strict "$target"
echo "installed $target ($(/usr/libexec/PlistBuddy -c 'Print :BBCommit' "$target/Contents/Info.plist"))"
