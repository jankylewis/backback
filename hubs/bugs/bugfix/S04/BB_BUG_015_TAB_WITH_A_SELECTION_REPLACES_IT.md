# BB-BUG-015 — fix

id: BB-BUG-015
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/edit/lines.rs`, `shell/desktop/src/editor/input/bb_keys.rs` (`text_key`)

## The change

`Document::indent` and `outdent`, mapped to Tab and Shift+Tab before the
plain-insert arm. With a selection, every line it touches moves in or out
one level (four spaces in; up to four spaces or one tab out) in one undo
step through `replace_many`, and the selection is put back with each end
moved by what was inserted or removed before it; a selection ending at a
line's start leaves that line alone, as VS Code does. Without a selection
Tab types spaces to the next stop and Shift+Tab outdents the cursor line.
A selection is never typed over.

## Proof

Red first: three unit tests (indent keeps the selection, one undo step;
outdent of spaces, a tab and bare lines; Tab to the next stop) and the
window test `tab_with_a_selection_indents_and_shift_tab_outdents`.
Readback on the installed app: `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
