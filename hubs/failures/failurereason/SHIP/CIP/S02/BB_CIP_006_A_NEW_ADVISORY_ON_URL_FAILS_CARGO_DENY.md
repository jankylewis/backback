# BB-CIP-006 — a new advisory on `url` fails `cargo deny` on CI

id: BB-CIP-006
family: CIP
status: OBSERVED
date: 2026-11-27
signature: RUSTSEC-2026-0592
source: CI on PR #133 (`bbeod-45-find-panel`), step "Dependency audit"
links: fix=yes lesson=no evidence=no toolgap=none

## What it is

The advisory RUSTSEC-2026-0592 was published upstream against `url` 2.5.8, the
version `bb_crates/Cargo.lock` named. 2.5.9 fixes it. CI's `cargo deny check`
failed on advisories for this PR. No backback code changed, and PR #133's code
did not cause it.

## How it shows

Only the dependency audit is red; every other check passes. The same family as
BB-CIP-003: CI fetches a fresh advisory database and index, so it sees what an
older local cache may not.

## How to confirm

`cargo deny check advisories` in `bb_crates/` names RUSTSEC-2026-0592 and `url`
2.5.8 in `Cargo.lock`.
