# BB-LSP-010 — lesson

id: BB-LSP-010
family: LSP
status: FIXED
source: BBEOD29, 2026-11-19

## Who believed what

Routing every request by language (BB-BUG-075) covered the side servers.

## What changed the belief

The failure text was never routed: a fake server failing on purpose
showed `C# server` for Python.

## Rule left behind

When one server becomes several, grep the old server's name in every
user text and route each one by the server that answered.
