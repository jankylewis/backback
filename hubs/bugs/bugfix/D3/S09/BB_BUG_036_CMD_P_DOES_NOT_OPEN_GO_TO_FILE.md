# BB-BUG-036 — fix

id: BB-BUG-036
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/open/mod.rs` (`bindings`)

## The change

Cmd+P is bound to Go to File as well; Cmd+Shift+O keeps its binding
until the headmate rules.

## Proof

Unit test `cmd_p_opens_go_to_file`. Readback on the branch build on
MbTx: Cmd+P opened Go to File with 5451 files listed (not loading) at
the first state 100 ms after the key; typing `pipe` filtered it, 246 us
p50 a key.

The first gate caught a slip: `cmd-p` inserted at index 1 broke
`cmd_o_and_the_file_menu_open_a_workspace`, which reads bindings by
position; `cmd-p` now comes last.
