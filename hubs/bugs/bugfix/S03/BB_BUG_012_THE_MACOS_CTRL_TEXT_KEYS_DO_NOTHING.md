# BB-BUG-012 — fix

id: BB-BUG-012
family: UIX
status: FIXED
source: `shell/desktop/src/editor/input/bb_keys.rs`, `bb_core/bb_text/src/document/movement/mod.rs`

## The change

With Ctrl held, the editor maps A/E/B/F/P/N/H/D to line start (column 0,
new `Document::line_start_move`), line end, char left/right, line up/down
and delete back/forward; Shift extends the moves. Any other Ctrl key is
not an edit (Ctrl+Space completion is unchanged).

## Proof

Red first: window test `ctrl_emacs_keys_move_and_delete` failed (Ctrl+E:
cursor 0, expected 3) and passes now; the 8 completion tests still pass.
