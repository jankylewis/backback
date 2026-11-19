# BB-UIX-003 — The gutter kept 50 px when zoomed, so line numbers ran into the code

id: BB-UIX-003
family: UIX
status: FIXED
date: 2026-11-02
signature: gutter stayed 50 px at zoom
source: `docs/plan/readback/FIXES/S02/BBEOD14/bb_READBACK.md`
observed: zoomed live capture: `1●namespace`, numbers and the diagnostic dot
  on top of the first column
links: fix=yes lesson=no evidence=yes toolgap=none

## What it is

The gutter width was a constant (`GUTTER_W = 50.0`), sized for 13 pt. At a
bigger size its number and dot overflowed into the text.

## How to confirm

Zoom in a few steps and look at the first column of each line.
