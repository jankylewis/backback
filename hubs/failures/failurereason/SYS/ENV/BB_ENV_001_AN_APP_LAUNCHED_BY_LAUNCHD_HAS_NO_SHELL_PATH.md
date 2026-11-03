# BB-ENV-001 — An app launched by launchd has no shell PATH

id: BB-ENV-001
family: ENV
status: FIXED
date: 2026-11-03
signature: PATH=/usr/bin:/bin:/usr/sbin:/sbin
source: `bb_crates/edge/dap/src/lib.rs` (`locate`), `bb_crates/bb_core/process/src/shell_env.rs`; bugs BB-BUG-008, BB-BUG-009
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-012

## What it is

launchd starts an app from the Dock or Finder with `PATH` =
`/usr/bin:/bin:/usr/sbin:/sbin` and no `TERM`. Everything the user's shell
adds (`~/.dotnet/tools`, `/opt/homebrew/bin`, `/usr/local/share/dotnet`,
`~/.cargo/bin`) is absent, so `dotnet`, `netcoredbg` and the Roslyn server
are not found, while the same app opened from a terminal inherits the shell
and works.

## How it shows

BB-BUG-008: F5 says `Debug failed: netcoredbg not found`. BB-BUG-009: a
project fails to load and `debug I/O: No such file or directory`, only when
the app is opened from the Dock. A readback with `open` from a terminal
never shows it: the shell leaks into the app (BB-TG-012).

## How to confirm it

`ps eww <pid>` of the running app: a `PATH` of four folders and no `TERM`
is a launchd start. From a terminal, `env -i HOME=.. USER=..
PATH=/usr/bin:/bin:/usr/sbin:/sbin <binary>` reproduces it.
