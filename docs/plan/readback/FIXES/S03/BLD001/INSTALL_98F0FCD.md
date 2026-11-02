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
