# BB-BUG-006 — Opened from the Dock or Finder, the app shows the disk root

id: BB-BUG-006
family: UIX
status: OBSERVED
date: 2026-11-01
signature: EXPLORER / No solution
source: `bb_crates/shell/desktop/src/app/args.rs:12` (folder or solution only from argv)
source: `bb_crates/shell/desktop/src/app/bb_launch.rs:69` (menus: no File > Open)
observed: headmate screenshot: Explorer root `/` (Applications, System, usr...),
  status bar "No solution"
links: fix=no lesson=no evidence=no toolgap=none

## Steps

Open backback.app from the Dock or Finder.

Expected: a welcome or the last solution, and File > Open (Cmd+O) to pick one.
Actual: the Explorer lists `/`; there is no way to open a solution from the app.

## Note

A Finder launch has no argument and its working folder is `/`. Goal: BBEOD13.
