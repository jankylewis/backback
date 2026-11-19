# BB-UIX-020 — fix

id: BB-UIX-020
family: UIX
status: FIXED
source: branch `fix/away-run`; `jump_to`

## The fix that holds

The mapped path decides the words: under `view.root` it is a local file
that no longer exists; elsewhere it is not on this machine.

## The wrong fix

Checking the parent folder: a deleted folder under the root would again
read as remote.
