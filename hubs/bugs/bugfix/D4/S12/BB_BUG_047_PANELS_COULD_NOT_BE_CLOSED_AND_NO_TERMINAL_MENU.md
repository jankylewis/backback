# BB-BUG-047 — fix

id: BB-BUG-047
family: UIX
status: FIXED
source: `intel/view/mod.rs` (`close_x`, `TITLE_PX`), `intel/view/panels.rs` (`close`), `run/bb_debug/view/ticks.rs` (`hide`), `terminal/{mod,view}.rs` (`hide`, `title`), `app/bb_launch/mod.rs` (`menus`)

## The change

One shared ✕ (`close_x`) at the right of every bottom panel's title row:

- Find in Files / References / Problems / rename preview: closes it, as Esc does.
- Debug: hides the panel, the session goes on; the next pause or F5 shows it again.
- Terminal: a `TERMINAL` title row with the ✕; hides it, the shell keeps
  running (Ctrl+` brings it back); the shell grid takes the title row off.

View → Terminal sends `ToggleTerminal` (the Ctrl+` action).

## Lesson

A panel that opens by a click must close by a click: a key-only close
(Esc) depends on focus the user cannot see.

## Proof

Window tests click each ✕: `find_in_files_results_close_on_their_cross`,
`the_terminal_closes_on_its_cross_and_the_shell_goes_on`,
`the_debug_panel_hides_on_its_cross_and_comes_back_on_a_pause`; unit
`menus_go_app_then_file_then_view_as_on_every_mac` now asks for View → Terminal. Each fails on 43e1686 (no ✕, no item).
