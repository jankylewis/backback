#!/bin/bash
# The full backback gate in one command (BB-TG-002, BB-TG-008): fmt, clippy,
# test, deny and bb audit, every step run, one exit code line each and one
# summary. Exit 0 only when every step passed.
set -uo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
# One build job and one test thread: more overheated the headmate's Mac
# (2026-11-11). CI sets its own workers in ci_tests.sh.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
# The debugger tests need netcoredbg (README: BACKBACK_NETCOREDBG or PATH).
default_dbg="$HOME/.local/share/backback/netcoredbg/netcoredbg"
if [ -z "${BACKBACK_NETCOREDBG:-}" ] && ! command -v netcoredbg >/dev/null; then
  if [ -x "$default_dbg" ]; then
    export BACKBACK_NETCOREDBG="$default_dbg"
  else
    echo "refused: netcoredbg not found; set BACKBACK_NETCOREDBG" >&2
    exit 2
  fi
fi
log="$(mktemp -t bb_gate)"
cd "$repo_dir/bb_crates" || exit 2

failed=0
step() {
  local name="$1"
  shift
  "$@" >>"$log" 2>&1
  local rc=$?
  echo "$name rc=$rc"
  [ "$rc" -eq 0 ] || failed=1
}

step fmt cargo fmt --check
step clippy cargo clippy --all-targets --all-features -- -D warnings
step test cargo test --workspace --all-features --no-fail-fast -- --test-threads=1
# The tests' processes have exited: remove the temp folders they left
# (BB-PRC-004). Live folders, another gate's too, are kept.
step sweep cargo run -q -p bb -- sweep
step deny cargo deny check
step audit cargo run -q -p bb -- audit ..

grep -E '^test result' "$log" |
  awk '{p += $4; f += $6} END {print "tests: suites=" NR " passed=" p " failed=" f}'
grep -E '^test .* FAILED$' "$log" | head -20
grep -E '^sweep ' "$log"
if [ "$failed" -eq 0 ]; then
  # A green gate's log holds nothing to read again: no file is left behind.
  rm -f "$log"
  echo "gate: green"
else
  echo "log: $log"
  echo "gate: RED"
fi
exit "$failed"
