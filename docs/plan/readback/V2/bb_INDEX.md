# Readback V2: units from 2026-11-03 on

One folder per unit (the `FIXES/` shelves S01 to S04 are full). Each folder
holds the steps, captures and verdict of a read back on the real app.
This folder is full (five entries); later units are in `../V3/`.

| Unit | What was read back |
|---|---|
| `BBEOD19/` | Exception breakpoints: a saved All tick pauses F5 on a caught throw, named in the panel and the status bar |
| `BBEOD20/` | Build before list: a never-built test project builds once on expand, then lists; `--no-write` refuses and says so |
| `BBEOD21/` | Dot-folders in search: Find in Files on MbTx equals `git grep`; Go to File lists `.github` files, never `.git`, `.idea` or ignored ones |
| `SWEEP01/` | Editor key sweeps 1 to 3 on the installed app (BB-BUG-013 to 019), the horizontal-scroll readback on the headmate's file, and the BB-BUG-019 fix read back before and after |
