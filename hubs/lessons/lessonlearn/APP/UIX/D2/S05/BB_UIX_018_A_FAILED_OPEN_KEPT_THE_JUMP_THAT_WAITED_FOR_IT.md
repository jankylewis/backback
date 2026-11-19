# BB-UIX-018 — lesson

id: BB-UIX-018
family: UIX
status: FIXED
source: BBEOD30 family sweep

## Who believed what

A jump stored for an open is used by that open.

## What changed the belief

The sweep for BB-BUG-080 clicked a row of a deleted file: the open
failed and the stored jump stayed for a later open.

## Rule left behind

Whatever waits for an async result is cleared on its failure arm too,
and a failure names the thing that failed.
