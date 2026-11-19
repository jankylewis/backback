# BB-BUG-080 — Back stayed live and did nothing after its file's tab closed

id: BB-BUG-080
family: UIX
status: FIXED
date: 2026-11-19
signature: ← is live, a click (or Cmd+[) does nothing
source: headmate report 2026-11-19 (BBEOD30); `bb_crates/shell/desktop/src/editor/bb_tabs/model/history/mod.rs` (`go`)
cause: BB-UIX-017
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open A.cs; jump inside it (F12, Go to Symbol), so Back points into
A.cs; close A.cs's tab (its last tab). ← stays live. Click it.

Expected (VS Code, headmate-approved): A.cs reopens at the place Back
kept; Forward, Cmd+[ / Cmd+], Ctrl+- / Ctrl+Shift+- the same. A place
whose file was deleted or renamed is skipped; with nothing reachable the
arrow is grey, and a key press says why in the status bar.

Actual: nothing happens. With another tab still open ← did reopen A.cs;
with no tab open the press returned early. A deleted file kept ← live
and its press failed with `file path: No such file or directory`.
