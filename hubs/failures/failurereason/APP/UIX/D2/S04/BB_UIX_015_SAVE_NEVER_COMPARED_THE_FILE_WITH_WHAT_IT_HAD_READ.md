# BB-UIX-015 — Save never compared the file with what it had read

id: BB-UIX-015
family: UIX
status: FIXED
date: 2026-11-04
signature: an outside change to an open file is lost at the next save
source: `bb_crates/bb_core/bb_text/src/document/io.rs` (`save`, before the guard)
observed: BB-BUG-034
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`save` resolved the target, kept its mode, owner and links, and wrote
the buffer. Nothing recorded what the file was when it was read, so a
newer file on disk could not be told from the one opened.

## How to confirm it

Open, change the file from outside, edit, save, read the disk.
