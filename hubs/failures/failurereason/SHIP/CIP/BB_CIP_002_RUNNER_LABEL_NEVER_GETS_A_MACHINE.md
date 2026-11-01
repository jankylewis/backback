# BB-CIP-002 — A runner label no hosted machine serves keeps the job queued

id: BB-CIP-002
family: CIP
status: FIXED
date: 2026-11-01
signature: gate:queued
source: `.github/workflows/bb_ci.yml` `runs-on`
observed: run 41323338161 (`runs-on: macos-26-arm64`) stayed queued 20+ min
  while `macos-latest` runs on other branches started in seconds; cancelled
links: fix=yes lesson=no evidence=yes toolgap=none

## What it is

GitHub queues a job until a runner with its label exists; a label that no
hosted image serves never starts and never fails.

## How to confirm

`gh run view <id> --json jobs`: status `queued`, no steps, long after a run
with `macos-latest` started.
