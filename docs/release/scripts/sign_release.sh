#!/bin/bash
set -euo pipefail

: "${BACKBACK_DEVELOPER_ID:?Set the Developer ID Application certificate name}"
: "${BACKBACK_NOTARY_PROFILE:?Set the notarytool keychain profile name}"
repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
app_dir="${1:-$repo_dir/.target/bundle/backback.app}"
archive="${app_dir%.app}.notary.zip"
release_zip="${app_dir%.app}.zip"

test -d "$app_dir"
codesign --force --deep --options runtime --timestamp --sign "$BACKBACK_DEVELOPER_ID" "$app_dir"
codesign --verify --deep --strict --verbose=2 "$app_dir"
ditto -c -k --keepParent "$app_dir" "$archive"
xcrun notarytool submit "$archive" --keychain-profile "$BACKBACK_NOTARY_PROFILE" --wait
xcrun stapler staple "$app_dir"
xcrun stapler validate "$app_dir"
ditto -c -k --keepParent "$app_dir" "$release_zip"
echo "$release_zip"
