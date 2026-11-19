# BB-BUG-080 — fix

id: BB-BUG-080
family: UIX
status: FIXED
source: branch `fix/nav-back-closed-file`; `editor/bb_tabs/model/history/` (`go`, `bb_stack.rs`), `edges/bb_title.rs`, `app/bb_launch/open/mod.rs`

## The change

A place is reachable while its tab is open or its file is on disk. Back
and Forward go with no tab open (a closed file reopens at its place),
drop unreachable places and name the file in the status bar; the arrows
are live only when a reachable place exists (`Tabs::can_go`, the same
rule the press uses). Ctrl+- / Ctrl+Shift+- added beside Cmd+[ / Cmd+].

## Proof

Model: `with_every_tab_closed_back_still_goes_to_a_file_on_disk_and_is_live`,
`back_skips_a_deleted_file_names_it_and_lands_on_the_next_place`,
`the_arrows_are_off_when_every_place_is_in_a_deleted_file_unless_its_tab_is_open`.
Window (mouse + keys): `back_by_mouse_reopens_a_closed_file_at_its_line_and_is_off_once_the_file_is_deleted`,
`with_no_tab_left_ctrl_minus_reopens_the_closed_file_at_its_line`.
All five red before, green after; mutations M1-M4, M8, M9 killed.
