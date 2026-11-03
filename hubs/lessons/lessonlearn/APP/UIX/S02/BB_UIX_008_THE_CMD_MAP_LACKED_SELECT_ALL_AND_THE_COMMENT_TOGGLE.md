# BB-UIX-008 — lesson

id: BB-UIX-008
family: UIX
status: FIXED
source: branch `bb-bug-016-017-select-all-comment`

## Who believed what

The Cmd keys the editor had were the ones users needed; a missing one
would be asked for.

## What changed the belief

Sweep 2 pressed the keys of VS Code's default macOS table one by one:
Select All and Toggle Line Comment were missing, and nobody had asked.

## Rule left behind

The reference editor's default key table is the checklist, not the list of
keys already added: every standard key is swept with a selection and
without, and a missing arm is a bug found by the sweep.
