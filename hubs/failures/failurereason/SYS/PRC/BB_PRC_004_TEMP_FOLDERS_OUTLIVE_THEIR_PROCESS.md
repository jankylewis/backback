# BB-PRC-004 — Temp folders outlive their process

id: BB-PRC-004
family: PRC
status: FIXED
date: 2026-11-03
signature: `$TMPDIR` holds `<prefix>-<pid>` folders of processes long gone
source: every `std::env::temp_dir().join(format!("...-{}", process::id()))`
observed: BB-BUG-027
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

App and test fixtures named a temp folder by their pid, and removed only
a folder of that same name before making it ("fresh copy"). A pid never
repeats in practice, so the remove never hit, and nothing removed the
folder at exit: shared fixtures live in a `static OnceLock` (never
dropped), the app quits without joining its worker. One folder per run,
forever; a built harness copy is 120-130 MB.

## How it shows

Disk use grows with use; nothing reports it.

## How to confirm it

List `$TMPDIR` for backback's prefixes and check each pid with `ps -p`.
