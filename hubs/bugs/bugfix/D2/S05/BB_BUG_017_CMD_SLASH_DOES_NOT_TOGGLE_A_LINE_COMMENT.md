# BB-BUG-017 — fix

id: BB-BUG-017
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/edit/lines.rs` (`toggle_line_comment`), `shell/desktop/src/editor/input/bb_keys.rs` (`command_key`)

## The change

`Document::toggle_line_comment`: the lines the selection touches (or the
cursor line) get `// ` after their indent, blank lines left alone; when
every such line already starts with `//`, the comment is removed instead.
One undo step through `replace_many`; the selection is put back by what
moved before each end. `command_key` maps Cmd+/ to it.

## Proof

Red first: `cmd_slash_comments_the_touched_lines_after_their_indent_and_back`
(unit: comment, uncomment, blank lines, one undo step, the cursor line) and
the window test. Readback on this build: sweep 2 case G8 (`goto:3:5 cmd-/ cmd-s`) saves `    // int alpha = beta + gamma;`; the other eight cases stay PASS. Cases in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
