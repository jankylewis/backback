# BB-LSP-005 — lesson

id: BB-LSP-005
family: LSP
status: FIXED
source: branch `bb-lsp-005-log-line`

## Who believed what

A log line tagged with the request name meant "the server logged our
request", so the test required it.

## What changed the belief

The tagged lines were lazy start-up side effects; a warm runner wrote none.

## Rule left behind

Assert only what an external server writes because of our own request;
print the real text of a passing run before choosing a log line to assert.
