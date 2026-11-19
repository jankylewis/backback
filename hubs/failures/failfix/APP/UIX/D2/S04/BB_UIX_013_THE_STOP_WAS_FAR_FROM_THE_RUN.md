# BB-UIX-013 — fix

id: BB-UIX-013
family: UIX
status: FIXED
source: branch `bb-bug-024-tests-panel`

## The fix that holds

The control that started a run turns into its stop while the run is on: ▶
becomes ■ on the spec's row, Run becomes ■ Stop on the class and project
rows; the toolbar's ■ Stop shows only during a run.

## The wrong fix

A bigger or red toolbar Stop: still away from where the user's eyes are,
and still shown when there is nothing to stop.
