# BB-BUG-067 — fix

id: BB-BUG-067
family: NAV
status: FIXED
source: `run/bb_debug/view/notes.rs` (`open_selected`), `run/mod.rs` (`root_actions`)

## The change

A stop (or a call-stack pick) in another file records the place it leaves; Back / Forward are window-wide actions; the new title bar shows ← → (BB-FEAT-090).

## Lesson

Every way the app moves you to another file is a jump: route it through the one place that records history.

## Proof

`back_returns_from_a_step_into_another_file_by_the_title_bar_or_the_key`: red without the record (mutation).
