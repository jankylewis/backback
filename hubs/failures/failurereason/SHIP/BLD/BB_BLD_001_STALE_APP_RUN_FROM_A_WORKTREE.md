# BB-BLD-001 — An app built before a fix keeps running from a worktree bundle

id: BB-BLD-001
family: BLD
status: FIXED
date: 2026-11-01
signature: app built before the fix is still running
observed: running binary `backback_wt/bbeod-07-release/.target/bundle/backback.app`,
  built 10:22, started 11:05; PR #9 merged 11:27 (+07)
observed: no backback.app in `/Applications`
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

There is no installed app. Each mate builds a bundle inside its own worktree,
and a user opens whichever one they find. That window never updates.

## How it shows

A bug fixed on `master` is still seen (BB-BUG-003). No Dock mascot either.

## How to confirm

`ps -o lstart,command` for the backback process, and the binary's mtime,
against the merge time of the fix.

## Fix

`docs/release/scripts/install.sh` builds from a clean `master` and installs
`/Applications/backback.app`; `BBCommit` in its Info.plist names the build.
See failfix `SHIP/BLD/BB_BLD_001_INSTALL_FROM_MASTER.md`.
