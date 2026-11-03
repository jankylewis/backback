# BB-BUG-023 — fix

id: BB-BUG-023
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/bb_explorer/panels/tests_panel/panel.rs` (`side_tabs`), `bb_crates/shell/ui/src/bb_theme.rs` (`chosen`)

## The change

One tab strip for both sides, `EXPLORER  TESTS` at the left. The chosen tab
takes the new theme token `chosen` (the pink of `ignored`, a token of its
own so the two meanings never couple); the other one takes the text colour.
A click on the other tab switches the side; on the chosen one, nothing.

## Proof

Window test `explorer_and_tests_sit_side_by_side_and_switch_the_panel`
(TESTS within 40 px right of EXPLORER; clicks switch both ways); unit test
`the_chosen_tab_is_pink_and_the_other_one_takes_the_text_colour`. The
colours were not read off the screen: no capture of the headmate's screen.
