# BB-TG-004 — The Apex CLI is not on nuget.org, so CI cannot run 9 ApexTestkit tests

id: BB-TG-004
for: BB-CIP-001
status: OBSERVED
date: 2026-11-01
source: `bb_crates/shell/bb_cli/tests/dotnet/bb_testing.rs` (`apex_command`)
source: `bb_crates/edge/dotnet/testkit/tests/common/mod.rs` (`apex_command`)
source: `.github/workflows/bb_ci.yml` (Tests `--skip`)
observed: CI runs 41325098190 and 41326263469: 9 tests panicked "apex.cli
  0.8.2-pre is not installed"; nuget.org returns 404 for `apex.cli`

`apex.cli 0.8.2-pre` comes from a private feed; this repo is public and
holds no credentials. CI skips by name exactly the 9 tests whose body calls
the Apex CLI helpers (3 in `bb_cli/tests/dotnet/bb_testing.rs`, 6 in
`testkit/tests/harness.rs`; found by a scan, listed in `bb_ci.yml`). The local
gate runs them.

Wanted: a public Apex CLI package, or a CI secret approved by the headmate.
