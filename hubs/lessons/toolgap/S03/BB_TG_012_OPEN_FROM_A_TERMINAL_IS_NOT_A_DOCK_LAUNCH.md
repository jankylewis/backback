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

Workaround: launch a user-like readback with launchd's environment:
`env -i HOME="$HOME" USER="$USER" SHELL=/bin/zsh PATH=/usr/bin:/bin:/usr/sbin:/sbin open -n -a <app> ...`.
