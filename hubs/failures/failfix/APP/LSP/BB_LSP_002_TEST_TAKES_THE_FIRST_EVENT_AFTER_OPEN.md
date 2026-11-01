# BB-LSP-002 — fix

id: BB-LSP-002
family: LSP
status: FIXED
source: `bb_crates/shell/desktop/src/intel/bb_worker/tests/mod.rs` (`first_diagnostics`)

## The fix that holds

`first_diagnostics` waits for the diagnostics after `Open`, skips only
`Progress`, and fails naming any other event (Stopped, Ready, Failed, Answer,
SyncFailed) - stricter output than before, same requirement. Used at all three
call sites.

## The wrong fix

Rerunning until green, or a longer timeout: the order, not the time, broke it.

## Proof

Full gate after the change (see the BBEOD12 PR). Not proven: which event came
first in the failing run (the old assert hid it).
