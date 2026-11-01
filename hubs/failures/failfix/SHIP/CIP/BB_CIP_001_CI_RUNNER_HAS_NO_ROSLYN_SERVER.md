# BB-CIP-001 — fix

id: BB-CIP-001
family: CIP
status: FIXED
source: `.github/workflows/bb_ci.yml` (Install pinned test tools)

## The fix that holds

CI installs the same tools as a dev machine, pinned: .NET SDK 10.0.403
(setup-dotnet pinned to a full sha), roslyn-language-server
5.13.0-1.26514.10 (`dotnet tool install --version`), netcoredbg 3.2.0-1092
(release zip, sha256 checked); `BACKBACK_ROSLYN_LS` / `BACKBACK_NETCOREDBG`
set; tool dirs cached. Tests run with `--no-fail-fast`, so every crate runs.

## The wrong fix

Skipping the Roslyn tests on CI, or making them pass when the server is
missing.

## Proof

The CI run of the BBEOD07-CI2 PR: all Roslyn worker tests pass on the hosted
runner; only the 9 Apex CLI tests are skipped by name (BB-TG-004).
