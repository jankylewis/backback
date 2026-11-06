#!/bin/bash
# CI only: run every test with parallel workers on the one macOS runner,
# as many as its RAM allows (headmate 2026-11-06: under the runner's RAM,
# never overloading it). The local gate stays one suite at a time.
# Arguments are libtest filters (--skip NAME), passed to cargo nextest.
set -euo pipefail

# Kept free for macOS, the runner agent and the build cache.
reserve_mb=2048
# One worker's worst case: a test process plus the Roslyn or dotnet test
# host it may start. Raise it if the peak below comes near the RAM.
per_worker_mb=1200

ram_mb=$(( $(sysctl -n hw.memsize) / 1048576 ))
cores=$(sysctl -n hw.ncpu)
workers=$(( (ram_mb - reserve_mb) / per_worker_mb ))
max_workers=$(( cores * 2 ))
(( workers > max_workers )) && workers=$max_workers
(( workers < 1 )) && workers=1
echo "runner: ${ram_mb} MB RAM, ${cores} cores -> ${workers} test workers"

# Build every test first, with all cores, so the workers only run tests.
cargo nextest run --workspace --no-run

# Sample the memory really in use every 2 s (active + wired + compressed
# pages, as Activity Monitor counts it; a sum of RSS counts shared pages
# many times) and the swap used; the peaks show the sizing is right.
page=$(sysctl -n hw.pagesize)
used_mb() {
  vm_stat | awk -v p="$page" '/^Pages active/ {a=$3} /^Pages wired down/ {w=$4}
    /^Pages occupied by compressor/ {c=$5} END {print int((a+w+c)*p/1048576)}'
}
swap_mb() {
  sysctl -n vm.swapusage | awk '{sub("M", "", $6); print int($6)}'
}
peak_file=$(mktemp)
echo "0 0" >"$peak_file"
(
  while :; do
    read -r peak_used peak_swap <"$peak_file"
    used=$(used_mb)
    swap=$(swap_mb)
    (( used > peak_used )) && peak_used=$used
    (( swap > peak_swap )) && peak_swap=$swap
    echo "$peak_used $peak_swap" >"$peak_file"
    sleep 2
  done
) &
sampler=$!
echo "memory in use before the tests: $(used_mb) MB, swap $(swap_mb) MB"

rc=0
cargo nextest run --workspace --no-fail-fast -j "$workers" -- "$@" || rc=$?
kill "$sampler" 2>/dev/null || true
read -r peak_used peak_swap <"$peak_file"
echo "::notice title=Test memory::peak ${peak_used} MB in use of ${ram_mb} MB, swap ${peak_swap} MB, with ${workers} workers"
exit "$rc"
