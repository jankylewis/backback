# BB-GAT-016 — CI ran only the missing-tool branch

id: BB-GAT-016
family: GAT
status: OBSERVED
date: 2026-11-20
signature: a variant test (`..._or_its_install_hint`, `..._or_names_the_install_command`) passes in CI in well under a second
source: `.github/workflows/bb_ci.yml` (installs Roslyn and netcoredbg only; no rust-analyzer, pyright or pytest)
observed: master CI run 44555640505 (2026-11-19): `pyright_serves_the_fixture_or_the_start_names_the_install_hint` 0.019 s, `pyright_checks_the_fixture_or_discovery_names_the_install_hint` 0.024 s, `pytest_lists_and_runs_pass_fail_skip_or_names_the_install_command` 0.265 s; every variant test passed
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Each variant test (BB-GAT-013) proves the real server when the tool runs
and the install hint when it does not. The CI runner had none of the
three tools and the Tests step did not set `BACKBACK_REQUIRE_TOOLS`, so
every PR proved only the hint branch: a broken real-server path could
merge green. Only the local gate ever ran the real branch.

## How to confirm

Read the Tests step's durations for the variant tests: a real pyright or
rust-analyzer start takes seconds; the hint branch takes milliseconds.
The install step's `before:` lines say which tools the image had.
