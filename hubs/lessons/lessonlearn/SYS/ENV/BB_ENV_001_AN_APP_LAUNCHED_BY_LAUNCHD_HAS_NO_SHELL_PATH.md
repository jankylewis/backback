# BB-ENV-001 — lesson

id: BB-ENV-001
family: ENV
status: FIXED
source: PRs #36, #37

## Who believed what

The installed app had been read back many times with `open -a` from a
terminal, so "the app finds dotnet" was taken as proven.

## What changed the belief

A readback with `env -i` and launchd's four-folder `PATH`: no `dotnet`, no
`netcoredbg`. `open` from a terminal passes the shell's environment on.

## Rule left behind

A user-like readback of a macOS app starts it the way the Dock does: `env
-i`, launchd's `PATH`, no `TERM`. Any tool the app needs is found from the
login shell or from the app's own install folder, never from whatever
environment it happened to inherit.
