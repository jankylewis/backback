# BB-BUG-045 — fix

id: BB-BUG-045
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/bb_tabs/model/` (`bb_groups.rs`, `keep.rs`), `editor/view/pane/` (`mod.rs`, `drag.rs`), `bb_tabs/bar/keys.rs`

## The change

- `Side { Left, Right }` became `Group(n)`, up to `MAX_GROUPS` = 3. Each
  group keeps its active tab and its pane's scroll list (no swap on
  focus); an empty group closes and the groups right of it move left.
- Panes share the editor by `Lines.shares`; each line between two panes
  drags their widths; a new pane count shares evenly again.
- Drop on a pane joins its group; the right half of the last pane makes a
  new group while fewer than three show. Ctrl+Cmd+Right / Left move one
  group; Cmd+\ moves right (VS Code's split), left from the third.
- Tab menu shows Move to Right Group and / or Move to Left Group.

## Proof

Model test `three_groups_close_up_when_the_middle_one_empties`; window
test `three_panes_share_the_editor_and_their_lines_drag` (thirds, the
second line drags only its two panes, Cmd+\ from the third moves left).
