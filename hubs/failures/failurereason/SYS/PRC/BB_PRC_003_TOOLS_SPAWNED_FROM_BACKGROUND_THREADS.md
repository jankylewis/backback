# BB-PRC-003 — `SN` Roslyn came from a niced readback launch, not the app

id: BB-PRC-003
family: PRC
status: OBSERVED
date: 2026-11-02
signature: SN roslyn-language-server
source: `docs/plan/readback/FIXES/S02/BBEOD13/bb_READBACK.md` (BB-PRC-002 family)
observed: readback launch `backback <harness> &` from zsh: app `SN`, Roslyn `SN`
observed: same build via `open -a /Applications/backback.app --args <harness>`:
  app `S` nice 0, Roslyn `S` nice 0
links: fix=no lesson=no evidence=yes toolgap=BB-TG-006

## What it is

The first reading blamed Roslyn being spawned from a background thread. That
was wrong: the app itself was `SN`. zsh's `BG_NICE` (on by default) renices a
job started with `&`, and every child inherits it. A user launch from Finder
or `open` is not niced, and Roslyn runs at normal priority.

## What it changes

No product fix is needed for Roslyn. BB-BUG-004 (Test Explorer) is not
explained by throttling; it stays OPEN. BB-PRC-002 was also read back from a
niced launch; its fix (spawn on the UI thread) holds by its own test.

## How to confirm

`ps -o stat,nice` of the app: `N` or nice > 0 means the launch shell niced it.
