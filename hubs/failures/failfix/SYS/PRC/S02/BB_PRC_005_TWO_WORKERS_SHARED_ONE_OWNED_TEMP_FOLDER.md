# BB-PRC-005 — fix

id: BB-PRC-005
family: PRC
status: FIXED
source: branch `night-readback-notes`

## The fix that holds

Anything that can run more than once in a process owns a suffixed folder
(`create_with`); the plain `create` is kept for one-per-process owners.

## The wrong fix

Dropping the remove-first step of `create`: two workers would then share
one scratch build and log and still remove it for each other on drop.
