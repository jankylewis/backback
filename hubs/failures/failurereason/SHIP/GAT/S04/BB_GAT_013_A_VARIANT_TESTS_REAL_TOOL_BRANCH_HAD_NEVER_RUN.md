# BB-GAT-013 — A variant test's real-tool branch had never run

id: BB-GAT-013
family: GAT
status: FIXED
date: 2026-11-19
signature: `pyright_checks_the_fixture_or_discovery_names_the_install_hint` ... `path: "/private/var/folders/`
source: `bb_crates/edge/toolchains/python/tests/bb_tools.rs` (`pyright_checks_the_fixture_or_discovery_names_the_install_hint`)
observed: first run with pyright 1.1.414 installed: red; the issues were right, the test's root was `/var/...` and pyright reports `/private/var/...`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A variant-aware test passes on either branch, so while the tool was
missing (BBEOD28) its present branch was never run, and a wrong
expectation in it stayed hidden. Here: the temp root was not
canonical. Not a product fault: window roots are canonical
(`app/args.rs` `start_for`) and tabs canonicalize every path.

## How to confirm

Set `BACKBACK_REQUIRE_TOOLS=1`: every missing-tool branch fails, so a
green run proves the real tools ran.
