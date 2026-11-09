# BB-BUG-060 — fix

id: BB-BUG-060
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/navigation.rs` (`reveal_again`), `editor/view/mod.rs` (`back_in_the_file`), `editor/bb_tabs/bar/bb_view.rs` (tab click)

## The change

`reveal_again` clears the once-per-file mark, so the next frame opens the
folders, selects the row and scrolls it into view. It runs on a tab click
(the active tab too) and on a press in the editor that takes focus back
from the tree or a panel; presses while typing in the editor do not move
the tree.

## Lesson

"Follow the active file" means the file in front of the user, not the
file that last changed. The first test only checked the selection; the
new one checks the row is inside the Explorer's bounds.

## Proof

`the_active_files_row_is_scrolled_into_the_explorers_view` (window test,
120 folders): red before (row 123 out of sight after a click back), green
after; red again when the tab-click reveal is removed.
