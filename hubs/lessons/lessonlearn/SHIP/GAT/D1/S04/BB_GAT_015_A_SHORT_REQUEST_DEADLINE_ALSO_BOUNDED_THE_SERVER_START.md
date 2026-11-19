# BB-GAT-015 — lesson

id: BB-GAT-015
family: GAT
status: FIXED
source: PR #118 CI, 2026-11-19

## Who believed what

`request_timeout` in a session config bounds the requests a test makes.

## What changed the belief

It also bounds `initialize`; the one-worker local gate started the
scripted server in time, four CI workers did not.

## Rule left behind

A test that measures a deadline gives the start its own; and a scripted
server's start is made slow on purpose so the coupling shows locally.
