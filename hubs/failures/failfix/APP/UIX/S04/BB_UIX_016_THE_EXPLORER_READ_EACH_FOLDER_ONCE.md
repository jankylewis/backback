# BB-UIX-016 — fix

id: BB-UIX-016
family: UIX
status: FIXED
source: branch `bb-bug-021-move-copy-lines`

## The fix that holds

Re-read loaded folders on the 2 s disk check and merge by path, so open
state survives.

## The wrong fix

Dropping every loaded folder's children on a tick: each expanded folder
would close, and the user's place in the tree would be lost.
