# BB-LSP-009 — fix

id: BB-LSP-009
family: LSP
status: FIXED
source: `Session::raw` asks again after -32801 within its deadline

## The fix that holds

One retry rule in the one funnel (`Session::raw`) every typed request uses:
200 ms pause, ask again while the deadline lasts, then the server's own
error. No caller retries on its own.

## Proof

`a_content_modified_reply_is_asked_again_and_the_hover_arrives` red
before, green after; `a_server_that_never_settles_ends_at_the_deadline_with_its_own_error`;
real rust-analyzer live test 8 of 8.

## The wrong fix

A retry inside the test's hover step: the test goes green while the app
still shows the error.
