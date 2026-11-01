# BBEOD12 readback: the cursor moves, the text never moves

Date 2026-11-01, by backback_mm. Bug BB-BUG-005, cause BB-UIX-001.

## Setup

- App: release `backback` built from branch `bbeod-12-cursor`, run with
  `--no-write` on a copy of `CapturedOutboundRequest.cs` (the headmate's file).
- Input through the window's own dispatch (`BB_KEYS`): `click:3:33` (a real
  mouse click, which focuses the editor), then `right`, `right`, `left` x4,
  `end`. `screencapture -l` of the app window at each `mark:`.

## Result

| Capture | Cursor | Line 3 text |
|---|---|---|
| `LINE3_OUT_BOUND.png` | `CapturedOut│bound` | one word, unchanged |
| `LINE3_OUTBO_UND.png` | `CapturedOutbo│und` | one word, unchanged |
| `LINE3_O_UTBOUND.png` | `CapturedO│utbound` | one word, unchanged |

Pixel check, line 3 crops (760 x 44 px, 2x display): between any two
captures only two 4 px stripes differ, the old and the new caret. Every other
pixel, all text included, is identical. One arrow key moves the caret about
15.6 px, one Menlo char at 2x.

| Step (keys) | Caret columns | Differing x ranges (px) |
|---|---|---|
| right (1) | 32 -> 33 | 523-526, 539-542 |
| right (1) | 33 -> 34 | 539-542, 554-557 |
| left x4 (4) | 34 -> 30 | 554-557, 492-495 |
| all (6) | 32 -> 30 | 523-526, 492-495 |

The caret moves 15-16 px per key and 62 px for four keys. Typing work for
the 7 keys: p50 73 us, p99 169 us per key.

## Not covered

- `end` did not move the cursor (the last two captures are identical): the
  End key is not bound in the editor: BB-BUG-007 (BBEOD10).
- Wide glyphs (emoji) and tabs are covered by the window tests, not this run.
