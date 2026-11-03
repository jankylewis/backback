# BB-TG-008 — a hand-typed gate missed the debugger path

id: BB-TG-008
for: none
status: FIXED
date: 2026-11-02
source: local gate of branch `bbeod-06-find`, 2026-11-02 23:50 +07

The gate for Find in Files was typed by hand without `BACKBACK_NETCOREDBG`.
netcoredbg is not on PATH on this Mac, so 14 debugger tests (`edge/dap`:
`lifecycle`, `reap`, `setnext`, `real_testhost`) failed with
`netcoredbg must be installed for this gate: NotFound`; 332 passed. The
change did not touch the debugger. Same family as BB-TG-002 (no one-command
gate): the gate depends on shell state a person must remember.

Fix: `docs/release/scripts/gate.sh` runs all five steps with PATH and the
debugger path set (default `~/.local/share/backback/netcoredbg/netcoredbg`,
refusal with exit 2 when none is found), one exit code per step and one
summary line. Closes BB-TG-002's "wanted" too.
