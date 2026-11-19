# BB-GAT-018 — fix

id: BB-GAT-018
family: GAT
status: FIXED
source: branch `test/late-langs-real`, the commit after 8fe9ec6

## The fix that holds

A hold lasts until released (cap 60 s, printed as `walk_hold_expired_ms` if ever hit). The relay drops the real link when the window's link closes, counts each server thread whose events end, and both real-server tests `wait_ended` after closing the window.

## The wrong fix

A longer sleep or `#[ignore]` on CI; a bigger cap alone (the LEAK stays); killing processes by name after the test.
