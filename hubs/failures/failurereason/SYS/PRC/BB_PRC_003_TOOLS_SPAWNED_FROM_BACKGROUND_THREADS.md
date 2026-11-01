# BB-PRC-003 — Roslyn and test runs are also spawned from background threads

id: BB-PRC-003
family: PRC
status: OBSERVED
date: 2026-11-02
signature: SN roslyn-language-server
source: `docs/plan/readback/FIXES/S02/BBEOD13/bb_READBACK.md` (BB-PRC-002 family)
observed: during the Build readback the app's Roslyn processes showed `SN`
related: BB-BUG-004 (Test Explorer did not respond; a throttled test host
  would look like that; not proven)
links: fix=no lesson=no evidence=no toolgap=none

## What it is

The same background-QoS spawn as BB-PRC-002 applies to the Roslyn server
and to test runs. Roslyn still works (slower load); a test run or debug
session may stall like the build did.

## How to confirm

Run one test from the Test Explorer and read `ps -o stat` of its host.
