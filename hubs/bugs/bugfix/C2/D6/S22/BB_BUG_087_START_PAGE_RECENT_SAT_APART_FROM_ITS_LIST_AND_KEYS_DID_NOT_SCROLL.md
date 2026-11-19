# BB-BUG-087 — fix

id: BB-BUG-087
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `recent_heading`, `StartPage::scroll`

## The change

`recent_heading` renders after LANGUAGES and the notes; `rows_box` tracks a `ScrollHandle` and `step` calls `scroll_to_item`.

A recent folder gone from disk stays as a greyed row, `no longer exists ·
<path>` (the note first, so a long path never hides it: seen on the served
app at 33aff15), with a × (or Delete) that drops it from the saved list
only, as VS Code keeps missing recents until removed. Nothing on disk is
touched (`recent::forget`).

## Proof

`the_recent_heading_sits_directly_above_its_list` red (RECENT y=74 above LANGUAGES y=119); `the_last_row_scrolls_into_view_by_keys_and_by_wheel` red (Open... at y=548, list 158-336); green after; mutations M4, M4b red. Gone rows: `a_gone_folder_shows_greyed_and_its_remove_button_drops_it_from_the_list_only`, `a_gone_rows_note_comes_before_its_path_so_a_long_path_never_hides_it` (red with the note after the path); mutations M4c, M4d red.
