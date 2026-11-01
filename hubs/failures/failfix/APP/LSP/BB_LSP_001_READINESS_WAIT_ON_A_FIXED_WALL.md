# BB-LSP-001 — fix

id: BB-LSP-001
family: LSP
status: OBSERVED
source: branch `bbeod-03-diagver` commit `28699aa`

## The fix

Count loaded projects by distinct path; wait on the server's readiness signal,
not a fixed wall time.

## The wrong fix

Raising the 30 s limit, or rerunning until green.

## Proof

Branch gate 289 passed / 0 failed. Not yet independently verified and not on
`master`, so status stays OBSERVED.
