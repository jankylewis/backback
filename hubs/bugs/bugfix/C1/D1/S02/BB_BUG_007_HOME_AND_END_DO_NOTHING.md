# BB-BUG-007 — fix

id: BB-BUG-007
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/movement/mod.rs` (`home`, `end`, `document_edge`)
source: `docs/plan/readback/FIXES/S01/BBEOD10/bb_READBACK.md`

## The change

Home: first non-blank, again: column 0 (smart). End: before the line break.
Cmd+Left/Right: the same; Cmd+Up/Down: document edges. Shift extends.

## Proof

Red first: 4 `Document` tests did not compile on master (no methods); window
test `home_end_and_cmd_arrows_move_the_cursor` through the editor keys.
Readback: End 3:46, Home 3:1, Cmd+Down 5:1, Cmd+Up 1:1 in the real app.
