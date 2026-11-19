# BB-UIX-020 — lesson

id: BB-UIX-020
family: UIX
status: FIXED
source: BBEOD30 note, BBEOD31 fix

## Who believed what

A stack path that does not resolve came from another machine.

## What changed the belief

BBEOD30's test-row sweep deleted a local file: the status blamed the
machine.

## Rule left behind

An error names the real reason; one error arm for two causes is split
by a fact the code already has (here the opened root).
