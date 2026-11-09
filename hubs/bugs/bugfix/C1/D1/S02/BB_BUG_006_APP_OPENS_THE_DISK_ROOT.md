# BB-BUG-006 — fix

id: BB-BUG-006
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/app/args.rs` (`resolve`)
source: `docs/plan/readback/FIXES/S02/BBEOD13/bb_READBACK.md`

## The change

With no argument and the working folder `/` (a Finder or Dock launch), the
app opens the newest recent workspace, else the home folder with the Open
panel. A terminal launch still opens its working folder. File > Open...
(Cmd+O) opens a folder, solution or file in a new window; each opened
workspace goes first in a versioned recent file.

## Proof

Red first: `a_finder_launch_never_opens_the_disk_root` (no `resolve` on
master). Readback: launched in `/`, the Explorer showed the recent harness.
