# BB-GAT-008 — fix

id: BB-GAT-008
family: GAT
status: FIXED
source: branch `fix/five-debug-window`

## The fix that holds

List every other variant (`other @ (Note::A | ...)`), as the session
tests do; run clippy on the touched crates before the gate.

## The wrong fix

`#[allow(clippy::wildcard_enum_match_arm)]` in the test: a new `Note`
variant would then pass every test unseen.
