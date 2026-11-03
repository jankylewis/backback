# BB-ENV-001 — fix

id: BB-ENV-001
family: ENV
status: FIXED
source: PRs #36 (BB-BUG-008), #37 (BB-BUG-009)

## The fix that holds

Two layers, as VS Code: an app without a `TERM` runs the login shell once
(`$SHELL -l -i -c`, 5 s, `PATH` read between markers) and adopts its `PATH`
before any thread or child starts; if the shell gives nothing, the .NET and
Homebrew folders are added. Tools backback installs are also looked up at
their own place: `~/.local/share/backback/netcoredbg`, then Homebrew.

## The wrong fix

Hard-coding folders alone (a user's `dotnet` can live anywhere the shell
knows), or telling the user to start the app from a terminal.

## Proof

Red first: `a_dock_launch_finds_netcoredbg_in_backbacks_install_folder`
failed with `NotFound`; the shell-env tests (no-terminal rule, markers with
banners, fallback, a real `/bin/sh -l`). Readback with `env -i` and
launchd's `PATH`: server Ready, Build succeeded, F5 pauses
(`docs/plan/readback/FIXES/S04/BB_BUG_008/`, `BB_BUG_009/`).
