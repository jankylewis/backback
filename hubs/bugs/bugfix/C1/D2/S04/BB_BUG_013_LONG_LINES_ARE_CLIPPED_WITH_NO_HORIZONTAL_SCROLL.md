# BB-BUG-013 — fix

id: BB-BUG-013
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/hscroll.rs`, `view/bb_line/mod.rs`, `input/mod.rs`, `bb_tabs/tab.rs`

## The change

A tab keeps `scroll_x`, the pixels its text has moved left. Each row puts
its shaped line in a clipped viewport after the gutter, shifted by that
much; the caret and the cursor popups shift with it; a click or hover adds
it back before the column lookup. A horizontal wheel or trackpad swipe
scrolls, clamped to the widest line in sight plus slack. A cursor move
keeps the caret in sight with a 24 px margin, as VS Code does; a caret that
fits in the first viewport shows the line start whole (Home on an indented
line). A swipe is never undone by the next paint: the view scrolls only
when the cursor itself moved. `state` prints `scroll_x`.

## Proof

Red first: `the_caret_stays_in_sight_on_a_long_line_and_a_swipe_scrolls`
(window): End on a 600-char line scrolls and the caret's drawn bounds lie
inside the editor; a click where the caret is keeps the cursor; Home
scrolls back to 0; a swipe scrolls 150 px, never past the longest line and
never left of the text. Unit: the scroll rule. Readback on the headmate's
own file, read-only: `docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
