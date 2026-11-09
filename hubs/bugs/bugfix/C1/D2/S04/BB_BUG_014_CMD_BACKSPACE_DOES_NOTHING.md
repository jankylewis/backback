# BB-BUG-014 — fix

id: BB-BUG-014
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/edit/lines.rs`, `shell/desktop/src/editor/input/bb_keys.rs` (`command_key`)

## The change

`Document::delete_to_line_start` and `delete_to_line_end`: the selection if
there is one, else the line from its start to the cursor (or the cursor to
its end); at the edge itself, the line break, so the line joins its
neighbour as in VS Code. `command_key` maps Cmd+Backspace and Cmd+Delete
to them.

## Proof

Red first: `cmd_backspace_and_cmd_delete_remove_to_the_line_edges` (unit)
and `cmd_backspace_and_cmd_delete_reach_the_line_edges` (window, through
the editor's own key handling). Readback on the installed app: `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
