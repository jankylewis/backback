# BB-BUG-025 — A click on a test does not open it

id: BB-BUG-025
family: UIX
status: FIXED
date: 2026-11-03
signature: a click on a test's row shows its result but not its code
source: the headmate's feedback 2026-11-03
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Click a test's name in the Tests panel.

Expected (VS Code's Test Explorer): the test's file opens on its method.
Actual: only the result pane changed; a failure's stack link was the one
way to the code, and a test that passed had none.

No failure mechanism: opening the source was not built.
