# BB-TG-005 — `screencapture -l` of a covered GPUI window returns a stale frame

id: BB-TG-005
for: BB-UIX-002
status: OBSERVED
date: 2026-11-01
source: `docs/plan/readback/FIXES/S01/BBEOD10/bb_READBACK.md` (Not covered)

In the BBEOD10 readback all window captures were byte-identical while the
model cursor moved (STATE_LOG). Most likely another window covered backback,
and a covered window is not redrawn, so the capture shows its last frame.
Not proven which window covered it.

Workaround: prove behaviour with `state` steps and the window list; take
pixel captures only while the backback window is in front (BBEOD12 run).
