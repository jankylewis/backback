# BB-UIX-002 — Minimize acted on `cx.active_window()`, None while another app is active

id: BB-UIX-002
family: UIX
status: FIXED
date: 2026-11-01
signature: min window_on_screen=23
source: `docs/plan/readback/FIXES/BBEOD10/bb_READBACK.md`
observed: first BBEOD10 readback build: after `cmd-m` the window list still
  showed the window; the app was not the active app
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-005

## What it is

The first Minimize handler was app-wide and looked up `cx.active_window()`.
That is the key window of the ACTIVE app; while another app is active it is
None, so Cmd+M sent to the backback window did nothing.

## How to confirm

Run the app while another app stays active; send `cmd-m` through `BB_KEYS`;
the window list still shows the window.
