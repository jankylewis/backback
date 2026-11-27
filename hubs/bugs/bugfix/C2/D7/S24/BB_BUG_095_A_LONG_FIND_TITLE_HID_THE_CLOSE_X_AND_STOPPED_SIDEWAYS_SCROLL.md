# BB-BUG-095 — fix

id: BB-BUG-095
family: UIX
status: OBSERVED
source: branch `bbeod-45-find-panel`

## The change

`intel/view/panels.rs`: the title div clips (`overflow_hidden`, `text_ellipsis`),
so a long title no longer sets the row's width; the `✕` stays in view.

## Proof

`a_long_title_keeps_the_cross_in_view_and_the_rows_scrolling` red at 487cb6f
(cross x=4035, panel 306..1920), green after: cross inside the panel, list no
wider than the panel, title and row text start at the same x, the row scrolls
300 px sideways, the `✕` closes it. Mutants: drop `overflow_hidden` (red).
