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

# Sample the memory in use every 2 s; the peak shows the sizing is right.
peak_file=$(mktemp)
echo 0 >"$peak_file"
(
  while :; do
    used=$(ps -A -o rss= | awk '{s += $1} END {print int(s / 1024)}')
    (( used > $(cat "$peak_file") )) && echo "$used" >"$peak_file"
    sleep 2
  done
) &
sampler=$!

rc=0
cargo nextest run --workspace --no-fail-fast -j "$workers" -- "$@" || rc=$?
kill "$sampler" 2>/dev/null || true
peak=$(cat "$peak_file")
echo "::notice title=Test memory::peak ${peak} MB of ${ram_mb} MB with ${workers} workers"
exit "$rc"
