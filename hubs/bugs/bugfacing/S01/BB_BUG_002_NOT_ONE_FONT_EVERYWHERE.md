# BB-BUG-002 — Explorer and side panels not in Menlo; sizes mixed

id: BB-BUG-002
family: UIX
status: FIXED
date: 2026-11-01
signature: Explorer rows not in Menlo
source: `bb_crates/shell/desktop/src/editor/view/bb_line.rs:118` (Menlo 13)
source: `bb_crates/shell/desktop/src/bb_explorer/bb_row_view.rs:66` (16 px, no font family)
observed: headmate screenshots: Explorer and Tests in the system font, editor in Menlo
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open a solution; compare the Explorer, Tests, tabs and editor text.

Expected: one font, Menlo 13 by default, on every surface.
Actual: Menlo only in the editor, panels and popups; literal sizes 10, 12, 13,
16 and 18 px across files.

## Note

Fonts and sizes are literals per view, so nothing keeps them equal. Goal:
BBEOD08 (one font token, a `bb audit` rule against literals, Settings window).
