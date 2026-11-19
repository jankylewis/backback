# BB-LSP-003 — Problems still counted a deleted error 6 s after the edit

id: BB-LSP-003
family: LSP
status: OPEN
date: 2026-11-01
signature: problems=18 after backspace
source: `docs/plan/readback/FIXES/S01/BBEOD03/STATE_LOG.txt` (run 1)
observed: 1 of 3 runs where the typing landed; load ~8-10
links: fix=no lesson=no evidence=yes toolgap=none

## What it is

After typing `zzz` (12 -> 18 problems) and deleting it, the count stayed 18
for at least 6 s in one run. In two other runs it went back to 12 within 6 s.

## Not proven

Whether the list was stale (an old version shown) or the server was slow, and
whether it would have settled: that run had no later check.

## How to confirm

Repeat the readback with `state` every second for 30 s after the deletion,
and log the document version of the shown diagnostics.

## Not reproduced on master 6dc536f (BBEOD35, 2026-11-20)

Window test by keys on the real Roslyn (`lang_tests/tools/live/roslyn.rs`,
`roslyn_problems_return_to_the_loaded_count_after_typed_text_is_deleted`):
`_Harness` copy, `GreeterSpec.cs`, `z z z` after `new Greeter();`, then
three backspaces, then the shown version, the last list sent and the count
each second for 30 s. 4 of 4 runs (1-min load 3.1-4.0): loaded 12, typed
14, the version 7 list (12) shown 1.02-1.03 s after the deletion and kept
for 30 s. At 0 s the window showed 0, not 18: the version 4 list is never shown on
version 7 text (BBEOD03-DIAGVER). Mutation: with no pull after a change
the test fails (no list for the edited version).

Still not proven: what run 1 of 2026-11-01 showed. Under the version rule,
18 at the shown version needs backspaces that never reached the text (run
`a` of the same readback lost its scripted click) or a list for the new
version computed on old text; neither is proven. Status stays OPEN until a
run shows it again or a ruling closes it.
