# BB-GAT-005 — lesson

id: BB-GAT-005
family: GAT
status: FIXED
source: the TESTS filter search branch

## Rule left behind

A wait in a test names the step that starts what it waits for, and comes
after it. A state that is only true once an action runs is read directly
before that action, never waited for.
