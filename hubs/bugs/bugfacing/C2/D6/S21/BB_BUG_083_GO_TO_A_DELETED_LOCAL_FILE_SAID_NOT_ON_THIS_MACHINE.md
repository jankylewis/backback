# BB-BUG-083 — Go to a deleted local file said "not on this machine"

id: BB-BUG-083
family: UIX
status: FIXED
date: 2026-11-19
signature: `Cannot open <root>/B.cs: not on this machine`
source: BBEOD30 note; `tests_panel/actions/navigation.rs` `jump_to` at 6f40103
cause: BB-UIX-020
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Run a failing test whose stack names B.cs in the opened folder; delete
B.cs; click the stack frame's link (a test row's "go to").

Expected: `Cannot open <path>/B.cs: the file no longer exists`.

Actual: `Cannot open <path>/B.cs: not on this machine`, as for a CI
path that was never local.
