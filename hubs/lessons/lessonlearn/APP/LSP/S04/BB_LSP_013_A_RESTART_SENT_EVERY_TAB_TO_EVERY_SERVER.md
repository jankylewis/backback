# BB-LSP-013 — lesson

id: BB-LSP-013
family: LSP
status: FIXED
source: BBEOD39 recording seam on the real servers

## Who believed what

"Resend every open tab" was right when Roslyn was the only server, so
the Restart path kept doing it after rust-analyzer and pyright arrived.

## What changed the belief

A seam that counts each `didOpen` per server start, on real servers.

## Rule left behind

A replay to a server that (re)started names that server; when routing
gains a server, sweep every "all tabs" loop that sends requests.
