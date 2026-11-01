# BB-CIP-001 — lesson

id: BB-CIP-001
family: CIP
status: FIXED
source: `.github/workflows/bb_ci.yml`

## Who believed what

A green local gate meant CI would be green.

## What changed the belief

The first CI run on `master` failed: the runner had no Roslyn server.
`cargo test` then stopped at that crate, hiding the next missing tool (the
Apex CLI) until `--no-fail-fast`.

## Rule left behind

CI installs every external tool the tests use, pinned and checked; run with
`--no-fail-fast` so one missing tool never hides the next.
