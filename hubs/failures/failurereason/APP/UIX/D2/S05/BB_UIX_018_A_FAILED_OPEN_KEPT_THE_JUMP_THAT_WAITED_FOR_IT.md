# BB-UIX-018 — A failed open kept the jump that waited for it

id: BB-UIX-018
family: UIX
status: FIXED
date: 2026-11-19
signature: `file path: No such file or directory (os error 2)`
source: `bb_crates/shell/desktop/src/main.rs` at bb82de2 (`open_file` and `finish_open` error arms: `self.report(&error)` only); `intel/glue/answers/mod.rs` (`goto_point` sets `intel.jump` before `open_file`)
observed: BB-BUG-081
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

A jump into a closed file is stored first (`intel.jump`,
`tests.pending_jump`) and run when the tab finishes opening. The open's
error arms only reported `TabError` (`file path: <io error>`, no file
name) and left the stored jump; `opened` runs a waiting jump on the next
open of that path.

## How to confirm it

Click a Places row of a deleted file; read `intel.jump` (still set) and
the status (no file name).
