# BB-GAT-013 — fix

id: BB-GAT-013
family: GAT
status: FIXED
source: `bb_tools.rs` canonical root; `BACKBACK_REQUIRE_TOOLS` in every variant test

## The fix that holds

The test uses the canonical root, as the session tests already did. Every
variant test (rust-analyzer and pyright sessions, rust-analyzer and
pyright discovery/check, pytest, the pytest and live window tests) fails
its missing-tool branch when `BACKBACK_REQUIRE_TOOLS` is set: a machine
that has the tools proves them by setting it.

## Proof

BBEOD29 runs with `BACKBACK_REQUIRE_TOOLS=1`: bb-python 7+6+3, bb-cargo
analyzer 6, bb-intel live sessions 3 (8 of 8 runs), window tools tests.

## The wrong fix

Rebasing pyright's paths onto a non-canonical root in the product.
