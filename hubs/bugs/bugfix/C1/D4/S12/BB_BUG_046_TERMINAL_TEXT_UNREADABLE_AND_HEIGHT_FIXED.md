# BB-BUG-046 — fix

id: BB-BUG-046
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/terminal/view.rs` (`colors`, `panel`, `grid`), `editor/view/pane/drag.rs` (`Edge::Terminal`)

## The change

The panel draws text in `theme.text` on `theme.bg` (`colors`), the font
and size are the window's (the editor's). Its top edge is a drag handle
(`Edge::Terminal`, `Lines.terminal_px`); the shell's grid is recomputed
from the new height.

## Lesson

A surface that sets its background must set its text colour too: gpui's
default text colour is black, which only reads on a light theme.

## Proof

`terminal_text_has_readable_contrast_in_both_themes` (WCAG >= 4.5 in DARK
and LIGHT; black text fails it); the window test drags the edge up 100 px
and the shell gets more rows.
