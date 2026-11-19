# BB-UIX-019 — lesson

id: BB-UIX-019
family: UIX
status: FIXED
source: BBEOD30 note, BBEOD31 fix

## Who believed what

A breakpoint lives as long as the user does not clear it.

## What changed the belief

BBEOD30's deleted-file sweep: the file can go while its breakpoints stay.

## Rule left behind

Every store keyed by a file path is checked against the disk where it is
used, and what is dropped is said.
