# BB-TG-009 — a readback launch takes the keyboard focus

id: BB-TG-009
for: none
status: OBSERVED
date: 2026-11-03
source: `bb_crates/shell/desktop/src/app/bb_launch/mod.rs` (`cx.activate(true)` at launch)

Each readback launch activates the app, so keys typed on the Mac's keyboard
at that moment go into the app under test. On 2026-11-03 00:48, two writable
edit runs on a scratch copy of the Greeter fixture got a stray letter at
line 1 (`f`, then `m`) and a backspace the script never sent; reruns with
the same keys were clean. It looked like an undo bug and was not.

Workaround: writable readbacks only on scratch copies; real solutions only
with `--no-write`; read a stray edit as possible outside input first.
