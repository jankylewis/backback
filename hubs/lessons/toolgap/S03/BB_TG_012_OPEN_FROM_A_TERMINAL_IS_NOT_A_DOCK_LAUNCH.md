# BB-TG-012 — `open` from a terminal is not a Dock launch

id: BB-TG-012
for: BB-BUG-009
status: OBSERVED
date: 2026-11-03
source: `ps eww` of the app started by `open -a` from the agent shell

`open` started from a shell passes that shell's environment to the app:
its `PATH` held the shell's entries (Homebrew, .NET, tool folders). A Dock
or Finder launch gets launchd's `/usr/bin:/bin:/usr/sbin:/sbin`. Readbacks
that "open the app like a user" from a terminal therefore hid every
missing-`PATH` bug.

Workaround: launch a user-like readback with launchd's environment, which
keeps `TMPDIR` and `LOGNAME` (read from a Dock-launched app with `ps eww`):
`env -i HOME="$HOME" USER="$USER" LOGNAME="$USER" SHELL=/bin/zsh TMPDIR="$TMPDIR" PATH=/usr/bin:/bin:/usr/sbin:/sbin <binary> ...`.
Without `TMPDIR`, Roslyn's MetadataAsSource (F12 into a framework type)
fails on a .NET named mutex and the C# server stops: an artifact of the
simulation, not a user bug (2026-11-03 04:47).
