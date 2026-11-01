# BBEOD08 + BBEOD14 readback: one font everywhere, zoom, Settings

Date 2026-11-02, by backback_mm. Release `backback` from `bbeod-08-font` on
a harness copy; `BACKBACK_STATE_DIR` a scratch folder.

## Real app

| Check | Result |
|---|---|
| Default size (after Cmd+0), live capture | Explorer, tab, editor, hover popup and status bar all in Menlo 13 (`MENLO_13_EVERYWHERE.png`); the popup was 12 px before |
| Zoom keys | `cmd-=` x4 -> settings `zoom=6`; `cmd-0` -> `zoom=0` |
| Zoom is saved | a new launch started at the saved `zoom=2` |
| Zoomed live capture (before the gutter fix) | every surface larger; the line-number gutter stayed 50 px and ran into the code: BB-UIX-003, fixed |

## Window tests (same code, every run)

- `zoom_scales_the_row_the_gutter_the_caret_and_the_text`: at 26 pt the row
  is the scaled line height, the text twice as wide, the caret after a
  100 px gutter.
- `the_settings_window_changes_size_family_and_side_live`: real clicks on
  the Settings window: size + -> 14, Monaco, Right, Reset -> Menlo 13 left.
- `no_surface_sets_its_own_font_family_or_text_size`: only the window root
  sets the font (BB-BUG-002 can not come back).

## Not covered

Later window captures were stale (the window was behind another window,
BB-TG-005), so the zoomed state after the gutter fix is proven by the window
test, not a capture. The macOS font list in Settings is not captured.
