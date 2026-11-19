# BB-BUG-087 — fix

id: BB-BUG-087
family: UIX
status: FIXED
source: branch `fix/readback-suspects`; `recent_heading`, `StartPage::scroll`

## The change

`recent_heading` renders after LANGUAGES and the notes; `rows_box` tracks a `ScrollHandle` and `step` calls `scroll_to_item`.

## Proof

`the_recent_heading_sits_directly_above_its_list` red (RECENT y=74 above LANGUAGES y=119); `the_last_row_scrolls_into_view_by_keys_and_by_wheel` red (Open... at y=548, list 158-336); green after; mutations M4, M4b red.
