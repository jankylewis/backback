# Triage — read first

One row per bug and per failure reason. Grep the signature in your log.

| Id | Signature | Status | First check |
|---|---|---|---|
| BB-BUG-001 | `Cmd+M does nothing` | OPEN | Window menu has a Minimize item bound to `cmd-m` |
| BB-BUG-002 | `Explorer rows not in Menlo` | OPEN | Every surface reads one font token, not a literal |
| BB-BUG-003 | `HARNESS Select an MCP server` | FIXED | Build time of the running binary vs `master` (BB-BLD-001) |
| BB-BUG-004 | `Test Explorer click no response` | OPEN | Reproduce by hand before blaming the input driver |
| BB-BUG-005 | `text after the cursor shifts right` | OBSERVED | Cursor is an overlay, never a char in the text runs (BB-UIX-001) |
| BB-BUG-006 | `EXPLORER / No solution` | OBSERVED | Finder launch has no argv; File > Open exists |
| BB-BLD-001 | `app built before the fix is still running` | OBSERVED | `ps -o lstart` and binary mtime vs merge time |
| BB-BLD-002 | `.app opens no window` | FIXED | `CFBundleExecutable` is the GUI binary; run `verify_bundle.sh` |
| BB-CIP-001 | `roslyn-language-server not found; set BACKBACK_ROSLYN_LS` | OBSERVED | `bb_ci.yml` installs the pinned Roslyn and netcoredbg |
| BB-LSP-001 | `before ready: ` | OBSERVED | Wait on readiness, count projects by distinct path |
| BB-PRF-001 | `RSS 1,008 MB with the 20 MB file` | FIXED | Heap profile: who holds the bytes before changing code |
| BB-UIX-001 | `push_run(&mut runs, '│'` | OBSERVED | Draw the cursor as a quad over the text |
