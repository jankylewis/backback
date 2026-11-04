# BB-BUG-047 — Panels had no close button; the View menu had no Terminal

id: BB-BUG-047
family: UIX
status: FIXED
date: 2026-11-04
signature: no ✕ on Find in Files, References, Problems, Debug or Terminal; View menu without Terminal
source: headmate request 2026-11-04 (build 43e1686: "plz support the closability of every panel"; "opening terminal by hitting View -> Terminal")
cause: not built: the panels closed only by keys (Esc with the editor focused; Ctrl+` for the terminal), and the View menu listed only theme and zoom
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Cmd+Shift+F, search, look for a way to close the results with the mouse;
open the View menu and look for Terminal.

Expected: a ✕ on each panel's title row; View → Terminal.
Actual: no ✕ (Esc closed results only while the editor had focus); no menu item.
