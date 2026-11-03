# BB-BUG-016 — fix

id: BB-BUG-016
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/edit/lines.rs` (`select_all`), `shell/desktop/src/editor/input/bb_keys.rs` (`command_key`)

## The change

`Document::select_all` selects from the first char to the end;
`command_key` maps Cmd+A to it. Typing then replaces the selection through
the insert path that already handles one.

## Proof

Red first: `select_all_takes_the_whole_text` (unit) and
`cmd_a_selects_all_and_cmd_slash_toggles_a_line_comment` (window).
Readback on this build: sweep 2 case G1 (`cmd-a x cmd-s`) saves `x`; the other eight cases stay PASS. Cases in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
