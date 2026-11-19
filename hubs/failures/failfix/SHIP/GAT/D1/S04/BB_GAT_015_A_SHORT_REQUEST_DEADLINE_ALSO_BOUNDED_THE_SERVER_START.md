# BB-GAT-015 — fix

id: BB-GAT-015
family: GAT
status: FIXED
source: `settle.rs`, `server_tests/bb_rust.rs`, `server_tests/python.rs`: `const START` (60 s) for `request_timeout`

## The fix that holds

The start (spawn and `initialize`) has its own generous deadline in every
session test config; only the request under test carries the short one.
The never-settles server now answers `initialize` after 2.5 s
(`SLOW_START`), longer than its request deadline, so the coupling cannot
come back unseen on a fast machine.

## Proof

Old coupling + slow start: red with `Lsp(Timeout { method: "initialize",
after: 2s })` (the CI text); fixed: settle 2 of 2, session tests 5 of 5
with `BACKBACK_REQUIRE_TOOLS=1`.

## The wrong fix

A longer request deadline in the never-settles test: it would stop
proving the request ends at its deadline.
