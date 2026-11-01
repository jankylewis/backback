# BB-UIX-003 — fix

id: BB-UIX-003
family: UIX
status: FIXED
source: `bb_crates/shell/ui/src/typography.rs` (`gutter_width`)

## The fix that holds

`Typography::gutter_width()` = size x 50 / 13 (50 px at 13 pt). The gutter
cell, caret, clicks, popup anchor and script hover all read it.

## Proof

`zoom_scales_the_row_the_gutter_the_caret_and_the_text`: at 26 pt the caret
sits after a 100 px gutter.
