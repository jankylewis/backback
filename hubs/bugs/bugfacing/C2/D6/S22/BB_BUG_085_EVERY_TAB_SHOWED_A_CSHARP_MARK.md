# BB-BUG-085 — Every tab showed a C# mark

id: BB-BUG-085
family: UIX
status: FIXED
date: 2026-11-19
signature: `C# main.py`, `C# main.rs` in the tab bar
source: BBEOD32 readback rows 2a, 3a; `editor/bb_tabs/bar/bb_view/mod.rs:172` at b13fcbb
cause: BB-UIX-022
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open `all3`; open `src/main.rs`, `tool/main.py`, `notes/README.md`, `App/Program.cs`.

Expected: Each tab shows its own language: `RS`, `PY`, none, `C#`.

Actual: Every tab shows `C# ` in front of its name: a literal from the C#-only app.
