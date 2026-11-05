# BB-BUG-054 — fix

id: BB-BUG-054
family: UIX
status: FIXED
source: `bb_crates/shell/ui/src/bb_theme.rs` (`selection`), `shell/desktop/src/editor/view/bb_line/mod.rs` (`text_run`)

## The change

A new theme token `selection`: #33446b dark (text on it 7.3:1), #cfe1e5
light (5.4:1). Selected editor text is the plain text colour on it,
whatever its syntax kind or link state; the terminal uses the same token.

## Lesson

A highlight colour picked for accents (bright, to stand out) is the wrong
background for text. Every background that text sits on gets a contrast
test against that text, in both themes.

## Proof

`selected_text_reads_and_the_selection_shows_in_both_themes` (bb-ui;
mutation-checked: the old accent fails at 1.29:1) and
`selected_text_is_plain_text_on_the_selection_colour` (mutation-checked:
syntax colour under a selection fails).
