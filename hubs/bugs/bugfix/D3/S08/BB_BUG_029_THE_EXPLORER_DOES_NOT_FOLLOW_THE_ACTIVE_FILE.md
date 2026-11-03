# BB-BUG-029 — fix

id: BB-BUG-029
family: UIX
status: FIXED
source: `bb_crates/bb_core/explorer/src/tree/bb_model.rs` (`Explorer::reveal`), `bb_crates/shell/desktop/src/bb_explorer/navigation.rs` (`side_panel`, `follow_active`)

## The change

`Explorer::reveal(path)` opens every folder down to the path (a project
of a solution counts by its folder) and gives its row. The side panel
calls it once per newly active file (`Tabs::revealed`), selects the row
and scrolls to it (`Nearest`). A folder the user closes after that stays
closed. The readback runner does it in its paint stand-in too (BB-TG-015).

## Proof

Unit tests `reveal_opens_every_folder_down_to_the_file_and_gives_its_row`,
`a_file_outside_the_tree_reveals_nothing`; window test
`the_explorer_selects_the_active_file`, red first with the call off.
Readback (branch build, `open:` step): `deep/a/b/walk.py` selected,
7 rows -> 10; then `pkg/app.py` selected, -> 12.
