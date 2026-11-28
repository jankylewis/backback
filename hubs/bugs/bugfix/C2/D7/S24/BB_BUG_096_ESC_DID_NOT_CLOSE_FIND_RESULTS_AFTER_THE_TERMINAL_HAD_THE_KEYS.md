# BB-BUG-096 — fix

id: BB-BUG-096
family: UIX
status: OBSERVED
source: branch `bbeod-45-find-panel`

## The change

`intel/view/panels.rs`: a left mouse-down on the panel focuses the editor
(`take_keys`), so the existing Esc path (`bb_keys::key`) closes the panel.

## Proof

`esc_closes_the_results_after_a_click_on_the_panel` red at 487cb6f ("Esc
closed it" failed), green after. Mutant: drop the `on_mouse_down` (red).
