# BB-UIX-002 — lesson

id: BB-UIX-002
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/bb_launch/bb_window.rs`

## Who believed what

"The active window" was taken to mean "the window the user is typing in".

## What changed the belief

The readback ran with another app active and Cmd+M did nothing.

## Rule left behind

A window command acts on the window the event reached, never on a global
lookup of the active window.
