# BB-PRF-002 — lesson

id: BB-PRF-002
family: PRF
status: FIXED
source: branch `bb-prf-002-goto`

## Who believed what

Go to File felt instant on every real solution (27 ms on Apex), so its walk
looked harmless.

## What changed the belief

The same keys on a tree at the feature's own cap (50,000 files) cost 204 ms
of UI work: the cost grows with the folder, and the cap allowed it.

## Rule left behind

Measure every feature at its own declared limit, not only on today's
solutions; anything that grows with the folder runs off the UI thread.
