# BB-GAT-009 — lesson

id: BB-GAT-009
family: GAT
status: FIXED
source: PR #114 CI run 44341920278

## Who believed what

A live debugger test green on every local machine would be green in CI.

## What changed the belief

CI runs the stock netcoredbg; only the local machines carry the patched
Set Next build, so the stock branch had never run anywhere.

## Rule left behind

A live test of an adapter feature that differs between stock and patched
asks the adapter's capabilities and asserts both branches; run it once
locally with `BACKBACK_NETCOREDBG` on the stock copy before handing over.
