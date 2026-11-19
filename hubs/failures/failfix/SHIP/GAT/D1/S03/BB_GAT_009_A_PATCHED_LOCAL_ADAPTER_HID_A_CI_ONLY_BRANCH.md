# BB-GAT-009 — fix

id: BB-GAT-009
family: GAT
status: FIXED
source: `session/tests/bb_async.rs` (`adapter_can_set_next`, `set_next_is_refused_and_the_arrow_stays`) on branch `fix/five-debug-window`

## The fix that holds

The test asks the adapter itself (a separate `DapSession` `initialize`)
whether it supports `gotoTargets`, prints `adapter_variant=`, then takes
one of two branches, both asserted, no early return:

- patched: the full proof, Set Next up twice and one step each;
- stock: Set Next ends in the visible `Refused` note with the exact
  text, one step runs on from the unmoved arrow to `await Task.Yield();`,
  and the late local `x = 1` still shows.

Both branches were run locally: stock via `BACKBACK_NETCOREDBG` on a
stock copy (installed adapter untouched), patched via the installed one.
Forcing the patched branch on the stock adapter gives the CI signature.

## The wrong fix

`#[ignore]`, an early `return`, or skipping when the adapter is stock:
CI would then prove nothing about the refusal a stock user sees.
