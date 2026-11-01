# BB-LSP-001 — fix

id: BB-LSP-001
family: LSP
status: FIXED
source: branch `bbeod-03-diagver` commit `28699aa`

## The fix

Count loaded projects by distinct path; wait on the server's readiness signal,
not a fixed wall time.

## The wrong fix

Raising the 30 s limit, or rerunning until green.

## Proof

Branch gate 289 passed / 0 failed (backback_1, loops 20/20 x3); rebased on
`master` 7ce4c96 with the MCP removal (3 harness projects, not 4) and a
full gate before merge (BBEOD03-DIAGVER PR).
