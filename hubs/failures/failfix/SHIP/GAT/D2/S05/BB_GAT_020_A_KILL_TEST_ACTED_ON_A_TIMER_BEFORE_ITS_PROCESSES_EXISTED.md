# BB-GAT-020 — fix

id: BB-GAT-020
family: GAT
status: FIXED
source: branch `bbeod-42-no-orphan-servers`, `app/script/mod.rs` `wait:go`, `tests/orphans/bb_tree.rs` `launch`

## The fix that holds

A script that ends backback itself holds at `wait:go` (bounded, 180 s) until
the file `BB_KEYS_GO` names exists; the test writes it only after it saw
every expected process. `launch` refuses a script with `crash:` or a quit
and no `wait:go`; the wait for processes stops at once, naming it, when
backback already ended. The kill tests (SIGKILL, SIGTERM, debug, unsaved
tabs) already acted from the test after the tree was seen.

## The wrong fix

A longer `sleep:` before the crash: still a race, only on a slower runner.

## Proof

Slow build, old keys: `servers missing ["sleep 600"] … printed true` (as CI);
`wait:go`: 9/9 green with the slow build. Without `go()`: "the panic ends
it"; old keys: refused at launch ("hold it at wait:go").
