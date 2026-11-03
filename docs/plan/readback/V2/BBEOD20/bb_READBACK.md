# BBEOD20 readback: build a never-built test project before listing it

Date 2026-11-03, by backback_mm. Branch `bbeod-20-build-before-list`. P4.

## What

Expanding a test project in the Test Explorer lists its tests with `dotnet
test --no-build`. On a fresh clone the assembly does not exist and the row
said `not built: ...dll (build the project first)`. Now, as VS Code and
Rider do, the list job builds the project once (`dotnet build`, the same
helper the debugger uses for a `.csproj`, 180 s per `dotnet` call) and lists
again; the row shows loading while it builds. With `--no-write` nothing is
built (a build writes `bin/` and `obj/`) and the row says
`not built: ... (--no-write: build the project outside backback)`.

## Result

| Check | Evidence |
|---|---|
| Never-built fixture, writes allowed: expand builds it (`bin/` appears) and lists 2 specs, no error | `a_never_built_project_is_built_on_expand_unless_no_write` (window) |
| Same fixture, `--no-write`: nothing built, the row names `--no-write` | same test, first pass |
| The built fixture still lists, runs a class and opens a failure as before | `test_panel_lists_runs_a_class_and_opens_a_failure` |
| Real app (branch build, `env -i` with launchd's PATH), scratch solution with no `bin/` or `obj/` anywhere: open the Tests panel, expand Calc.Tests | `state` after 50 s: `bb_tests_state projects=1 discovered=2 loading=0`; `Calc.Tests/bin/Debug/net10.0/Calc.Tests.dll` exists afterwards (built by the app); 62 s from launch to quit |
| Same, before this change (master binary): the row stayed `not built: ...dll (build the project first)` | the message the P4 proposal quotes |

## Not covered

An Apex project: the Apex CLI lists through its own command and never
reports `not built`.
