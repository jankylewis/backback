# BB-TG-003 — gpui `closest_index_for_x` returns the line end past the last glyph

id: BB-TG-003
for: BB-UIX-001
status: OBSERVED
date: 2026-11-01
source: gpui `text_system/line_layout.rs` `closest_index_for_x` (zed rev ba63a02)

For any x past the start of the LAST glyph, gpui returns the line end, so a
click on the left half of the last char lands after it. Seen as the red
`a_click_lands_on_the_column_drawn_under_the_mouse` (col 38, want 37).

Workaround: `Columns::column_at` compares x with the middle of the last glyph.
Also: layout snaps element x to whole device pixels and rounds a text box's
width up, so caret and width tests allow half a pixel and under one pixel.
