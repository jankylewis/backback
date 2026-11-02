# BB-PRF-002 — Go to File walked the whole root on the UI thread

id: BB-PRF-002
family: PRF
status: FIXED
date: 2026-11-03
signature: Go to File open freezes the window on a big folder
source: `bb_quick_open/view.rs` `show` called `QuickOpen::new(root)`, which ran `walk`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Cmd+Shift+O listed every file under the root (up to 50,000) on the UI thread
before the box could draw.

## How it shows

UI work per key (`BB_KEYS`, release build): Apex (about 1,200 listed files)
p99 21 ms, max 27 ms; a 50,000-file tree p99 196 ms, max 204 ms: about 12
frames lost each time the box opens. Typing in the open box stays at
p99 5.5 ms.
