# BB-BUG-059 — fix

id: BB-BUG-059
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/recent.rs` (`add`, `KEEP`)

## The change

Saving the list drops folders that are gone from disk, so they give
their place up; the list keeps 10 (was 8). Shipped in PR #91.

## Lesson

A list that hides entries when shown must also stop counting them when
saved; check a cap against what the user can see.

## Proof

`a_path_goes_first_once_and_the_list_keeps_ten_still_on_disk`: a deleted
folder is dropped and 10 are kept.
