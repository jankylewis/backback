# BB-UIX-012 — fix

id: BB-UIX-012
family: UIX
status: FIXED
source: branch `bb-bug-024-tests-panel`

## The fix that holds

Every head is a row with a `Group` key, its marker from
`TestsState::is_collapsed`, its click on `TestsState::toggle`.

## The wrong fix

Dropping the ▾ from the labels: honest, but it leaves a long suite with no
way to hide what the user is not looking at.
