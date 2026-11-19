# BB-LSP-009 — lesson

id: BB-LSP-009
family: LSP
status: FIXED
source: BBEOD29 live runs, 2026-11-19

## Who believed what

Every JSON-RPC error from a server is a failure to show; the BBEOD27 live
test only waited on definition, so its hover never met a reload.

## What changed the belief

The real rust-analyzer, run 6 times, answered one hover with -32801.

## Rule left behind

Live tests on a real server run more than once before they count; LSP
error codes that mean "ask again" are handled in the client, never shown.
