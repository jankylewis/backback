# BB-BUG-028 — fix

id: BB-BUG-028
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/input/clip.rs`

## The change

Cmd+C and Cmd+X take the selection, else the cursor's whole line with its
break (VS Code). Cmd+V pastes over the selection or at the cursor; a
whole line copied with no selection goes in above the cursor's line.
Cut and paste are refused in a read-only tab, as other edits are.

## Proof

Window test `cmd_c_copies_the_selection_and_cmd_v_pastes_it` (real
keystrokes, the test platform's clipboard): red first with the handler
off (clipboard `None`), green with it. Three unit tests on the span and
the line paste.
