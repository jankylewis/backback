# BB-DAP-001 — lesson

id: BB-DAP-001
family: DAP
status: FIXED
source: branch `bb-dap-exception-breaks`

## Who believed what

The DAP spec's `exceptionId` and `description` were the exception's type and
message, so the headline joined them.

## What changed the belief

The real reply: netcoredbg uses `exceptionId` for a category path and
`description` for a sentence; the bare values sit in `details`.

## Rule left behind

Print the adapter's real reply before shaping any text a user reads; a spec
field's name says where a value may be, not what it holds.
