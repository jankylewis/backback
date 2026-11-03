# BB-BUG-019 readback: diagnostics follow an edit in another file (2026-11-03)

Hand-check solution (`Calc`, `Calc.Tests`), a fresh scratch copy per run,
one binary per run: the installed app at `c145669` (before) and the branch
build at `6d945f5` (after). Every run asks one question (Shift+F12) before
it reads intel state, so the open and the edits reach the server whatever
the display does (BB-TG-015).

## R1: rename `Half` to `Halve`, restored copy

Keys: `goto:21:23 f2`, four Backspaces (the old box appends, BB-BUG-018),
`Halve`, Enter (preview), Enter (apply), Cmd+S, `goto:7:29 shift-f12`,
5 s, `state`; then Go to File `MathOpsTests`, Enter, `state`.

| Build | `problems` | Program.cs | MathOpsTests.cs | Files on disk |
|---|---|---|---|---|
| before `c145669` | 1 | v2, no squiggle | v1, squiggle at `18:25` (`MathOps.Halve`) | both `Halve`, no `Half(` |
| after `6d945f5` | 0 | v2, no squiggle | v1, no squiggle | both `Halve`, no `Half(` |

The squiggle at 18:25 is the call to the new name, judged against the old
Program.cs: the rename opened MathOpsTests.cs (its first pull) before the
server had Program.cs's edit, and only Program.cs was pulled again.

## R2: `dotnet restore` outside the app, unrestored copy

Open MathOpsTests.cs, `goto:16:25 shift-f12`, 3 s, `state`; the script
runs the restore once that line is printed; 45 s later `state` again, no
key in between.

| Build | Restore | `problems` before | 45 s after |
|---|---|---|---|
| before `c145669` | rc 0, 0.6 s (`-p:NuGetAudit=false`) | 10 (NUnit unresolved) | 10, the same squiggles |
| after `6d945f5` | rc 0, 0.7 s | 10 (NUnit unresolved) | 0, no squiggle |

The first before-run restore, without `-p:NuGetAudit=false`, hung for its
whole 120 s timeout; the network answered on IPv4 and IPv6 right after. The
rerun skipped the audit's network call. The flag changes how the packages
are checked, not what the restore writes for the app to see.

## Why the first "fresh load 0" proved nothing

Four runs on the installed app that asked no question (writable and
`--no-write`; `state` 2 s and 6 s after Ready; MathOpsTests.cs on a
restored copy): `bb_diag_state version=None` in 4 of 4. The tab never
reached the server, so `problems=0` measured nothing (BB-TG-015).

## Installed at `123c493` (the fix merged, #56)

| Check | Result |
|---|---|
| R1 rename, restored copy | `problems=0`, no squiggle in either file; both read `Halve` |
| R2 restore outside the app (`-p:NuGetAudit=false`) | 10 -> 0, no key in between |
| BB-LSP-003 probe: `zzz` before `System.Console`, then deleted | 0 -> 1 (`9:9`) -> 0 at +1 s, still 0 at +10 s (one run) |
| Sweep 2 (G1-G9) and sweep 3 (H1-H6) | all PASS |

BB-LSP-003 stays OPEN: it failed one run in three under load ~8-10; one
clean run on a quiet machine does not close it.

## Verdict

PASS on the branch build: an edit in one file and a restore outside the
editor both reach every open file's list. Before the fix both stay stale.
Red test: `an_edit_in_one_file_refreshes_the_diagnostics_of_the_other_open_files`
fails after 94.6 s on master's server loop and passes in 5.2 s with the fix.
