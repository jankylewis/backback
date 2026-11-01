# BB-CIP-001 — The CI runner has no roslyn-language-server or netcoredbg

id: BB-CIP-001
family: CIP
status: OBSERVED
date: 2026-11-01
signature: roslyn-language-server not found; set BACKBACK_ROSLYN_LS
source: `.github/workflows/bb_ci.yml` (no install step for either tool)
observed: GitHub Actions run 41273164163 on `master` (PR #9 merge): 3 tests in
  `intel::bb_worker::tests` FAILED with the signature above
links: fix=no lesson=no evidence=no toolgap=none

## What it is

Local machines have both tools; the CI runner does not. Tests that start the
real Roslyn server fail before ready. CI on `master` is red since PR #8.

## Fix still to make

Install pinned dotnet, roslyn-language-server and netcoredbg (sha256 checked)
in `bb_ci.yml`; run tests with `--no-fail-fast`.
