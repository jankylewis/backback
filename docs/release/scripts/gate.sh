#!/bin/bash
# The full backback gate in one command (BB-TG-002, BB-TG-008): fmt, clippy,
# test, deny and bb audit, every step run, one exit code line each and one
# summary. Exit 0 only when every step passed.
set -uo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
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
step test cargo test --workspace --all-features --no-fail-fast -- --test-threads=2
step deny cargo deny check
step audit cargo run -q -p bb -- audit ..

grep -E '^test result' "$log" |
  awk '{p += $4; f += $6} END {print "tests: suites=" NR " passed=" p " failed=" f}'
grep -E '^test .* FAILED$' "$log" | head -20
echo "log: $log"
[ "$failed" -eq 0 ] && echo "gate: green" || echo "gate: RED"
exit "$failed"
