# BB-BUG-044 — fix

id: BB-BUG-044
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/pane/drag.rs`, `pane/mod.rs` (`area`, `layout`), `intel/view/panels.rs`

## The change

- One drag model, `Lines`, in place of the tool window's `resizing` flag:
  the tool window edge, the line between the panes (the left pane's
  share), and the top edge of the panel under the editor (its height;
  the debug panel shares it). A drag moves from its press point.
- The right pane's width is kept apart (`right_px`), so text scrolling
  sideways stays right when the share is not half.
- Results panel: the match first, then `path:line:col` dimmed; rows keep
  their full width in a list that scrolls both ways.

## Proof

Window tests: the divider drag (left +60, right -60), the tool window
edge, the panel's top edge (+80 px), a 400-char row wider than the list,
the path after the match, and a sideways wheel that scrolls the list. A
mutation (stretched rows) fails the scroll test.
