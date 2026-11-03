# BB-UIX-002 — fix

id: BB-UIX-002
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/bb_window.rs` (`on_view`)

## The fix that holds

Minimize and Zoom are handled on the root view, with the `Window` the action
was dispatched to. Hide / Hide Others / Show All stay app-wide (they act on
the app, not a window).

## Proof

The rerun of the BBEOD10 readback: `min window_on_screen=none`.
