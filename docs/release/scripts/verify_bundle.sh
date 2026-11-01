#!/bin/bash
set -euo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
app_dir="${1:-$repo_dir/.target/bundle/backback.app}"
app_dir="$(cd "$app_dir" && pwd)"
screenshot="${2:-}"
executable="$(/usr/libexec/PlistBuddy -c 'Print CFBundleExecutable' "$app_dir/Contents/Info.plist")"
test "$executable" = backback || { echo "wrong bundle executable: $executable" >&2; exit 1; }
test -x "$app_dir/Contents/MacOS/backback"
echo 'bundle executable: backback GUI binary'

bundle_pids() {
  ps -axo pid=,command= | awk -v executable="$app_dir/Contents/MacOS/backback" '{pid=$1; $1=""; sub(/^[[:space:]]+/, ""); if ($0 == executable || index($0, executable " ") == 1) print pid}'
}
test -z "$(bundle_pids)" || { echo 'this bundle already has a process' >&2; exit 1; }
fixture_dir="$(mktemp -d /tmp/backback-bbeod07.XXXXXX)"
printf 'public class Example {}\n' > "$fixture_dir/Example.cs"
cleanup() {
  if test -n "${process_id:-}"; then kill -TERM "$process_id" 2>/dev/null || true; fi
  rm -rf "${fixture_dir:?}"
}
trap cleanup EXIT
open -n -a "$app_dir" --args "$fixture_dir"
process_id="$(bundle_pids)"
test -n "$process_id" || { echo 'bundled app process did not start' >&2; exit 1; }
case "$process_id" in *$'\n'*) echo 'more than one bundle process started' >&2; exit 1;; esac
window_id=''
for attempt in {1..30}; do
  window_id="$(BACKBACK_VERIFY_PID="$process_id" swift -e 'import CoreGraphics; import Foundation; let pid = Int(ProcessInfo.processInfo.environment["BACKBACK_VERIFY_PID"] ?? "-1") ?? -1; let windows = CGWindowListCopyWindowInfo([.optionOnScreenOnly, .excludeDesktopElements], kCGNullWindowID) as? [[String: Any]] ?? []; if let window = windows.first(where: { ($0[kCGWindowOwnerPID as String] as? Int) == pid && ($0[kCGWindowLayer as String] as? Int) == 0 }), let id = window[kCGWindowNumber as String] as? Int { print(id) }' 2>/dev/null)"
  if test -n "$window_id"; then
    break
  fi
  sleep 1
done
test -n "$window_id" || { echo 'no visible backback window' >&2; exit 1; }
echo "served window: $window_id"
if test -n "$screenshot"; then
  screencapture -x -l "$window_id" "$screenshot"
  test -s "$screenshot"
  echo "window screenshot: $screenshot"
fi

BACKBACK_VERIFY_PID="$process_id" swift -e 'import AppKit; import Foundation; let pid = Int32(ProcessInfo.processInfo.environment["BACKBACK_VERIFY_PID"] ?? "-1") ?? -1; guard let app = NSRunningApplication(processIdentifier: pid), app.terminate() else { exit(1) }'
for attempt in {1..10}; do
  if ! kill -0 "$process_id" 2>/dev/null; then
    echo 'leftover processes: 0'
    exit 0
  fi
  sleep 1
done
echo 'bundled app process remained after quit' >&2
exit 1
