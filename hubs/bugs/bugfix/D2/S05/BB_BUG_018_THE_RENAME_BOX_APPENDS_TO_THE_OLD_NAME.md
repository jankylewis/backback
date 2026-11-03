# BB-BUG-018 — fix

id: BB-BUG-018
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/intel/model/popups.rs` (`RenameInput::fresh`), `intel/glue/bb_keys.rs` (`rename_box`), `intel/view/bb_popups.rs`

## The change

The box opens with the old name marked `fresh`, drawn as a selection. The
first typed char replaces the whole name; Backspace clears it; from then
on the box edits as before. Enter still previews, Enter again applies.

## Proof

Red first: `the_box_opens_with_the_old_name_selected_so_typing_replaces_it`
(window): `f2 N e w enter` asks for `New`, not `xNew`. The existing rig
(`f2 backspace T o t a l enter`) still asks for `Total`. Readback: sweep 3
H6 on this build types `Halve` with no Backspace and both files read
`Halve`; cases in `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
