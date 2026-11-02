# BB-BUG-008 readback: F5 in the app opened as a user opens it

Date 2026-11-03, by backback_mm.

## Before (installed app at `55fac1d`)

`open -a /Applications/backback.app` on a scratch copy of the Debuggee
solution, no `BACKBACK_NETCOREDBG` in its environment (an app opened from
the Dock, Finder or `open` inherits launchd's, not a shell's). F9 on line
18, F5: "Debug failed: netcoredbg not found; set BACKBACK_NETCOREDBG or
install netcoredbg (MIT) and put it on PATH"; no session.

## After (bundle from `bb-bug-008-debugger-path`)

Same launch with `open -n <bundle>`, the shell's environment holding no
`BACKBACK_NETCOREDBG` either; netcoredbg in
`~/.local/share/backback/netcoredbg`.

| Step | Window state |
|---|---|
| F5 | paused (breakpoint): `Program.Square():18`, `Program.Main():10`; n=1, result=0 |
| F10 | paused (step): `Square():19`; result=1 |
| Shift+F5 | "Debugging ended: stopped"; no netcoredbg left |
