# BB-LSP-005 — fix

id: BB-LSP-005
family: LSP
status: FIXED
source: branch `bb-lsp-005-log-line`

## The fix that holds

The test asserts what the server writes for a request of ours: after the
load, the diagnostics pull still answers (its `unwrap`) and the server's
`[info] Shutting down`, its reply to our shutdown, reaches the log. No line
that only a Roslyn side effect writes is required.

## Proof

Local: printed the log after the load on a green run; the
`[textDocument/diagnostic]` lines were the BuildHost start and the
file-based reload. The new assertions hold on that log and on the CI log of
run 41501823925 (shutdown line present). 3 of 3 local runs green.
