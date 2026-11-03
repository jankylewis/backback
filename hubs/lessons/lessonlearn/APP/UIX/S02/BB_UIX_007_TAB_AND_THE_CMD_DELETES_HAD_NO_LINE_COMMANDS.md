# BB-UIX-007 — lesson

id: BB-UIX-007
family: UIX
status: FIXED
source: branch `bb-bug-014-015-line-keys`

## Who believed what

Keys the editor did not name could fall through to "type the char over the
selection", since that is what a letter does.

## What changed the belief

A sweep that pressed every editing key with a selection: Tab deleted the
selected line and left a tab; Shift+Tab the same.

## Rule left behind

A key with a selection is its own case. Every editing key is listed with
what it does to a selection (replaces it, extends it, or acts on its
lines) before the editor is called done, and the default insert arm takes
printable chars only. When a sweep finds one such key, the whole key table
is swept.
