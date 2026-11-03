# BBEOD24 readback: Show Call Hierarchy (2026-11-03)

Branch build of `bbeod-24-call-hierarchy`, `--no-write`, a fresh scratch
copy of the harness fixture per case. `state` lines are 1-based; the new
`bb_places_state` line prints the places panel's title and rows.

| Case | File, keys | Result |
|---|---|---|
| C1 callers of `Hello` | Core/Greeter.cs, `goto:5:19 shift-alt-h` | PASS: `bb_places_state title="1 caller of Greeter.Hello(string)" places=["GreeterSpec.cs:14"]` |
| C2 callers of `Greets` | _NUnitApex/GreeterSpec.cs, `goto:11:17 shift-alt-h` | PASS: `message=Some("No callers found")`, no panel |

Live (real roslyn-language-server, `edge/intel/tests/goto.rs`): the server
advertises `callHierarchyProvider: Some(Simple(true))`; prepare names
`Greeter.Hello(string)` on line 4 (0-based); its one caller is
`GreeterSpec.Greets(string, string)`, one call site on line 13 holding the
call's `Hello`.

The window test installs the app's own key bindings (`bind_keys`) and
sends Shift+Alt+H as macOS does (key `h`, char `Ó`): the ask goes out and
no `Ó` is typed (BB-UIX-010).

## Verdict

PASS. One caller lists rather than jumps; no caller says so.
