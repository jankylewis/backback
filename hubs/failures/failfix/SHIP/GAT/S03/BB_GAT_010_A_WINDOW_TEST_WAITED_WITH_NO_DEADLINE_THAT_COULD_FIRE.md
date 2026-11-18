# BB-GAT-010 — fix

id: BB-GAT-010
family: GAT
status: FIXED
source: `lang_tests/watchdog.rs`; `rust_panels.rs`, `tests_panel/view/window_tests.rs`, `bb_debug/tests.rs` (each wait helper arms it)

## The fix that holds

A watchdog thread armed by every real-time wait in window tests: past its
deadline it prints `waited <time> for <what>` and ends the run (exit
101), even while the test thread is blocked. The waits also fail by
themselves at their own deadline with what the window showed. The two
test causes were fixed: a per-row `test-project-<name>` selector, and
`bind_keys` before a key test.

## Proof

`a_blocked_wait_ends_the_run_at_its_deadline_naming_what_it_waited_for`
(a child run that blocks): red on the stub (ran 15 s past its 1 s
deadline), green with the watchdog (1.1 s).

## The wrong fix

A longer deadline, or a timeout only around `cargo test`: the run still
sits silent until the outer limit.
