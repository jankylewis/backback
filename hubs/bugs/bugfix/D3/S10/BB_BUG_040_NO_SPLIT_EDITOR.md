# BB-BUG-040 — fix

id: BB-BUG-040
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/bb_tabs/model/` (`bb_groups.rs`, `keep.rs`), `editor/view/pane/mod.rs`

## The change

- One tab list, each tab tagged Left or Right (left run first), so every
  walk over all tabs (C# server, rename, disk watch, quit prompt) is
  unchanged. A file is open once: opening it again shows it in its group.
- `active` is the focused group's tab, `other_active` the other's; each
  pane has its own scroll list, swapped on focus.
- Drag a tab: onto the right half of a lone pane it splits; with two
  panes, onto a pane it joins that group. Menu rows Move to Right / Left
  Group; keys Ctrl+Cmd+Right / Left, and Cmd+\ to the other group (VS
  Code's Split Editor copies the editor; here it moves it, one tab per file).
- A group with no tabs left closes; the split ends.
- The other pane draws its active tab; its first press focuses it and
  then places the cursor as usual. Click maths add the left pane's width.
- Tab bulk closes and pins act inside the tab's own group.

## Not done

The other pane does not move the parse window of a file over 4 MB while
it scrolls; its lines past that window show without highlight.

## Proof

Model tests `model/tests.rs`; window tests `view/pane/window_tests.rs`
(a real drag onto the right half, a press in the other pane, Cmd+\). A
mutation (no focus on press) made the press test fail.
