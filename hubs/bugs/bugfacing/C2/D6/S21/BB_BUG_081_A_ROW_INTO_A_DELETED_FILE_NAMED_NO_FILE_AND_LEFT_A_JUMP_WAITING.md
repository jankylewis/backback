# BB-BUG-081 — A row into a deleted file named no file and left a jump waiting

id: BB-BUG-081
family: UIX
status: FIXED
date: 2026-11-19
signature: `file path: No such file or directory (os error 2)`
source: BBEOD30 family sweep of BB-BUG-080; `bb_crates/shell/desktop/src/main.rs` (`open_file`, `finish_open` error arms)
cause: BB-UIX-018
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Find in Files (or references, Problems) lists a line of B.cs. Delete
B.cs outside the app (or make it unreadable) and click the row; later
bring B.cs back and open it from the Explorer.

Expected: `Cannot open B.cs: <why>`; the later open starts at the top.

Actual: `file path: No such file or directory (os error 2)` (no file
named), and the row's jump stayed stored (`intel.jump`); `opened` runs a
stored jump on the next open of that path, so the later open lands on
the old row's line (code read; the test asserts the stored jump). A
test row's "go to" on a file that resolves but cannot be read keeps its
`tests.pending_jump` the same way.
