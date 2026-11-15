# BB-GAT-007 — Audit fixes went to the gate unbuilt

id: BB-GAT-007
family: GAT
status: FIXED
date: 2026-11-15
signature: gate output `clippy rc=101`, `test rc=101`, `tests: none run`; `no method named on_action found for struct gpui::Div`
source: the split terminal branch (self-detected)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The audit asked for shorter functions and a shorter `main.rs`. The action
registration moved into `terminal::actions`, which calls `on_action` on a
`Div` without importing `InteractiveElement`. The audit passed; the gate
was started without a build of the refactored tree, and it failed to compile.
A second run then hit BB-GAT-004 again: View got "Split Terminal" and
the menu test still counted 8.
