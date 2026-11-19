# BB-GAT-010 — A window test waited with no deadline that could fire

id: BB-GAT-010
family: GAT
status: FIXED
date: 2026-11-19
signature: test binary at 0 % CPU for minutes, no children, stack in `wait_until` (`thread::sleep`)
source: `shell/desktop/src/app/bb_launch/open/lang_tests/rust_panels.rs` (`wait_until`, BBEOD27 draft)
observed: the matemaster saw the BBEOD27 test run idle 5+ min and stopped it; the seat's `sample` showed both stuck tests in `wait_until`
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

Two new window tests waited for a state that could never come (a click
on a shared `test-project` selector hit the member with no tests; a
Cmd+Shift+B with no key bindings installed built nothing). The wait had a
300 s deadline and gave no detail, and a deadline checked between
`run_until_parked` calls cannot fire while one runs a blocked job (real
cargo or dotnet, up to 30 min in the product's own limit).

## How to confirm it

Run each test alone with `perl -e 'alarm N; exec @ARGV'`; a stuck one is
killed by the alarm with no output. Sample it: the stack is in the wait.
