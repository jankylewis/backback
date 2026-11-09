# BB-BUG-008 — fix

id: BB-BUG-008
family: SHIP
status: FIXED
source: `bb_crates/edge/dap/src/lib.rs` (`locate`, `locate_in`)

## The change

After the override and `PATH`, `locate` also looks where an app opened from
the Dock can still find it: `~/.local/share/backback/netcoredbg`, then
Homebrew's `/opt/homebrew/bin` and `/usr/local/bin` (as the Roslyn locator
adds `~/.dotnet/tools`). The error names the install folder; so does the
README.

## Proof

Red first: `a_dock_launch_finds_netcoredbg_in_backbacks_install_folder`
(a Dock-like `PATH`, no override) failed with `NotFound`. Readback: the
installed app opened with `open` and no variable pauses on the breakpoint
(`docs/plan/readback/FIXES/S04/BB_BUG_008/`).
