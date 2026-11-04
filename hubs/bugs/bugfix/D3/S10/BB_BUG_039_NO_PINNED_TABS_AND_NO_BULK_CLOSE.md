# BB-BUG-039 — fix

id: BB-BUG-039
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/bb_tabs/bar/` (`mod.rs` model, `bb_view.rs` tab bar, `menu.rs` menu and keys)

## The change

- `Tab.pinned`; `toggle_pin` moves a tab to the end of the pinned run (or
  just after it); the active file stays active.
- `close_set(at, CloseSet)`: Others / Right / Left / Unpinned skip pinned
  tabs, All takes every tab. Clean tabs close at once; dirty ones queue,
  and the save prompt asks about each in turn, showing it; Cancel ends
  the whole close.
- A right-click opens the tab menu; a pinned tab's × is a pin that unpins.
- Keys as in VS Code: Cmd+K Shift+Enter pin / unpin, Cmd+Alt+T Close
  Others, Cmd+K Cmd+W Close All.
- Readback: runner step `tabmenu:<n>:<row>`, state line `bb_tabs_state`.

## Proof

Unit tests in `bar/tests.rs` (pin order, the skip rules, the dirty queue
and Cancel). Window tests in `bar/window_tests.rs`: a real right-click
opens the menu and its rows act; the Cmd+K Shift+Enter chord pins and
inserts no line.
Readback (branch build, scratch files, `--no-write`): pin by menu, the
chord, close right / left / except pinned / all, and the dirty-tab prompt,
6 of 6 as expected. Its first run caught a slip in the new runner step:
`tabmenu:<n>` made `n` 0-based twice (`number` already does), so it acted
on the tab before; fixed before the PR.
