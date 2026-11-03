# BB-BUG-028 — Cmd+C does not copy

id: BB-BUG-028
family: UIX
status: FIXED
date: 2026-11-03
signature: a selected phrase, Cmd+C, and the clipboard is unchanged
source: the headmate's report 2026-11-03 (a document number selected in `_RevisionDocRepo.cs`)
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Select text in the editor, press Cmd+C, paste elsewhere.

Expected: the selection is on the clipboard; Cmd+X and Cmd+V work too.
Actual: nothing: the editor had no copy, cut or paste at all (only the
terminal pasted). The key sweeps of BB-UIX-010 covered line commands, not
the clipboard.

No failure mechanism: not built.
