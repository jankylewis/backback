#!/bin/bash
# The backback gate (BB-TG-002, BB-TG-008, BB-TG-023): one exit code line per
# step and one summary; exit 0 only when every step passed.
#
#   gate.sh          what the branch changed: fmt and audit always; clippy and
#                    tests for each changed crate and every crate that uses
#                    it; the whole workspace when Cargo.toml, Cargo.lock or
#                    .cargo/ changed. Docs and hubs only: fmt and audit.
#   gate.sh --full   the whole workspace (CI runs it on every PR).
#   gate.sh --plan   print what would run, run nothing.
#
# One build job and one test thread (BB-ENV-002).
set -uo pipefail

repo_dir="$(cd "$(dirname "$0")/../../.." && pwd)"
export PATH="$HOME/.cargo/bin:$PATH"
# One build job and one test thread: more overheated the headmate's Mac
# (2026-11-11). CI sets its own workers in ci_tests.sh.
export CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}"
full=0
dry=0
for arg in "$@"; do
  case "$arg" in
    --full) full=1 ;;
    --plan) dry=1 ;;
    *) echo "refused: unknown option $arg (--full, --plan)" >&2; exit 2 ;;
  esac
done
cd "$repo_dir/bb_crates" || exit 2

# The crates to check, one per line; FULL for the whole workspace; nothing
# when no crate changed. Changes: the branch against its base, plus the
# working tree.
plan() {
  [ "$full" -eq 1 ] && { echo FULL; return; }
  local base="${BB_GATE_BASE:-origin/master}"
  local changed
  changed="$({ git -C "$repo_dir" diff --name-only "$base"...HEAD
    git -C "$repo_dir" diff --name-only HEAD
    git -C "$repo_dir" ls-files --others --exclude-standard; } 2>/dev/null | sort -u)"
  cargo metadata --format-version 1 2>/dev/null |
    CHANGED="$changed" REPO="$repo_dir" python3 -c '
import json, os, sys
meta = json.load(sys.stdin)
repo = os.environ["REPO"]
changed = [os.path.join(repo, p) for p in os.environ["CHANGED"].split("\n") if p]
root = os.path.join(repo, "bb_crates")
if any(p.startswith(os.path.join(root, ".cargo")) or p in
       (os.path.join(root, "Cargo.toml"), os.path.join(root, "Cargo.lock")) for p in changed):
    print("FULL"); sys.exit()
members = {pid: pkg for pkg in meta["packages"] for pid in meta["workspace_members"]
           if pkg["id"] == pid}
dirs = {pid: os.path.dirname(pkg["manifest_path"]) + os.sep for pid, pkg in members.items()}
hit = set()
for path in changed:
    owners = [pid for pid, d in dirs.items() if path.startswith(d)]
    if owners:
        hit.add(max(owners, key=lambda pid: len(dirs[pid])))
users = {pid: set() for pid in members}
for node in meta["resolve"]["nodes"]:
    for dep in node["deps"]:
        if node["id"] in members and dep["pkg"] in users:
            users[dep["pkg"]].add(node["id"])
todo = list(hit)
while todo:
    for user in users[todo.pop()] - hit:
        hit.add(user); todo.append(user)
for pid in sorted(hit, key=lambda pid: members[pid]["name"]):
    print(members[pid]["name"])
'
}

crates="$(plan)"
if [ "$dry" -eq 1 ]; then
  case "$crates" in
    FULL) echo "plan: the whole workspace" ;;
    "") echo "plan: no crate changed: fmt and audit only" ;;
    *) echo "plan: $(echo "$crates" | tr '\n' ' ')" ;;
  esac
  exit 0
fi

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
case "$crates" in
  FULL)
    echo "scope: the whole workspace"
    step clippy cargo clippy --all-targets --all-features -- -D warnings
    step test cargo test --workspace --all-features --no-fail-fast -- --test-threads=1
    step deny cargo deny check
    ;;
  "")
    echo "scope: no crate changed (fmt and audit only)"
    ;;
  *)
    picks=()
    while IFS= read -r crate; do picks+=(-p "$crate"); done <<<"$crates"
    echo "scope: $(echo "$crates" | tr '\n' ' ')"
    step clippy cargo clippy "${picks[@]}" --all-targets --all-features -- -D warnings
    step test cargo test "${picks[@]}" --all-features --no-fail-fast -- --test-threads=1
    ;;
esac
# The tests' processes have exited: remove the temp folders they left
# (BB-PRC-004). Live folders, another gate's too, are kept.
step sweep cargo run -q -p bb -- sweep
step audit cargo run -q -p bb -- audit ..

grep -E '^test result' "$log" |
  awk '{p += $4; f += $6} END {
    if (NR == 0) print "tests: none run"
    else print "tests: suites=" NR " passed=" p " failed=" f
  }'
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
