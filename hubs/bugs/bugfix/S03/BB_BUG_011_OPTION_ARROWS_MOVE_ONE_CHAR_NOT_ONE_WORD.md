# BB-BUG-011 — fix

id: BB-BUG-011
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/movement/words.rs`, `shell/desktop/src/editor/input/bb_keys.rs`

## The change

`Document::word_left/word_right` (with Shift: extend) and
`delete_word_back/delete_word_forward`. Stops as in VS Code / IntelliJ:
the end (start) of the next (previous) word or punctuation run; a line
break is a stop of its own (`\r\n` whole, BB-BUG-010); an indent that runs
into a line edge stops at the edge. With a selection, the deletes remove
only the selection. The editor maps Option+Left/Right/Backspace/Delete.

## Proof

Red first: 6 `Document` tests (stops, line breaks, Shift, deletes,
selection) failed on stubs; the window test
`option_arrows_and_option_backspace_work_by_word` failed on the old key
map and passes on the new one. Readback in the release app: see the PR.
