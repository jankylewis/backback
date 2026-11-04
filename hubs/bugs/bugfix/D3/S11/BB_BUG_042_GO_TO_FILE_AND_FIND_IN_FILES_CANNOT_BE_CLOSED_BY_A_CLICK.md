# BB-BUG-042 — fix

id: BB-BUG-042
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/bb_quick_open/view.rs` (`finish`, `rows`, `overlay`)

## The change

One `finish(accept)` closes the box for Escape, Enter, a press outside
(`on_mouse_down_out`) and a row press (selects that row, then opens it).
History records a place only when something opens.

## Proof

Window test `a_press_outside_closes_the_box_and_a_row_press_opens_its_file`;
with the outside handler removed it fails.
