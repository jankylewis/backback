# BB-UIX-015 — fix

id: BB-UIX-015
family: UIX
status: FIXED
source: branch `bb-bug-021-move-copy-lines`

## The fix that holds

Stamp (mtime, length) at read and at write; refuse once on a mismatch,
overwrite on the second ask.

## The wrong fix

Refusing every save over a newer file: the user's edits could never be
written without a reopen that throws them away.
