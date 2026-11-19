# BB-LSP-013 — fix

id: BB-LSP-013
family: LSP
status: FIXED
source: `resync_open(view, started)`; `restart` and `languages_found`

## The fix that holds

`resync_open` takes the servers that just started and reopens only their
tabs. Its callers `flush` before starting the servers, so every other
tab's pending open, close and edit goes to its own running server, and a
tab that landed but was not flushed is opened once (by the resync).
Restart passes `[Roslyn]`; the walk's answer passes all three.

## Proof

Restart test red before (`[(0, cs), (1, rs), (2, py), (3, cs)]` with two
"already open" failures), green after. Mutations, each red: no server
filter; no flush in `restart`; no flush in `languages_found`.

## The wrong fix

Ignoring `AlreadyOpen` in the worker: the second open is still sent, and
the dropped edits of the other tabs stay lost.
