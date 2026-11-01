# BB-BUG-001 — fix

id: BB-BUG-001
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/bb_window.rs`
source: `docs/plan/readback/FIXES/BBEOD10/bb_READBACK.md`

## The change

A Window menu (Minimize, Zoom) and app-menu items Hide Backback, Hide Others,
Show All. Keys: Cmd+M minimize, Cmd+H hide, Cmd+Alt+H hide others. Minimize
and Zoom act on the window the key reached (BB-UIX-002).

## Proof

Red first: `cmd_m_minimizes_cmd_h_hides_and_cmd_alt_h_hides_others` and
`the_app_menu_can_hide_and_a_window_menu_minimizes_and_zooms` did not compile
on master (no actions). Readback: after Cmd+M the window list no longer shows
the window.
