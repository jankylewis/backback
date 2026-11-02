# Install readback at 98f0fcd: the app as a Dock user gets it

Date 2026-11-03 04:30 +07, by backback_mm. `docs/release/scripts/install.sh`
on a clean `master` at `98f0fcd`: `installed /Applications/backback.app
(98f0fcd)`; `BBCommit` in Info.plist is `98f0fcd`.

## Launch as the Dock does (BB-TG-012)

`env -i HOME USER SHELL=/bin/zsh PATH=/usr/bin:/bin:/usr/sbin:/sbin open -n
-a /Applications/backback.app` on a scratch copy of the Debuggee solution.
`ps eww`: the app started with `PATH=/usr/bin:/bin:/usr/sbin:/sbin`;
`ps -o stat,nice`: `S 0` (normal priority).

| Step | Window state |
|---|---|
| C# server | Ready, no failed project (BB-BUG-009 fixed) |
| Cmd+Shift+B | "Build succeeded: 0 warnings" |
| F9 on line 18, F5 | paused (breakpoint): `Program.Square():18`, `Program.Main():10`; n=1, result=0 (BB-BUG-008 fixed) |
| F10 | paused (step): `Square():19`; result=1 |
| Shift+F5 | "Debugging ended: stopped"; no netcoredbg left |

## In this build since the last install (55fac1d)

BB-BUG-008 (#36), BB-BUG-009 (#37), BBEOD18 clickable stack traces (#34),
BB-CIP-003 yanked crate (#35).

## Not covered

Mouse-only checks (screen locked, BB-TG-010): see proposal P3 in
`docs/plan/bb_roadmap/PROPOSALS.md`.

## Real solutions, read-only (`--no-write`), launchd environment

Direct exec of the installed binary with `env -i` (BB-TG-010 addendum).
Apex: Find in Files `ToolOutcome` 141 results (= `git grep`) in 1,236 files,
207 ms while the C# server loads 27 projects; UI work p99 6.3 ms. Go to
Symbol in `PageTools.cs`: `close` -> line 59 `CloseAsync`, `activ` -> line 43
`Activate`. MbTx: Find `SearchKey` 326 results in 3,588 files, 295 ms;
`git grep` 327, the one line is in a tracked hidden folder, skipped by design
(proposal P5); Go to File `readme` + Enter opens `README.md`. Both trees'
`git status` unchanged. A scratch Greeter copy: type, Cmd+S, delete, Cmd+S:
the file equals the original byte for byte.
