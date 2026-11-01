# BB-BUG-003 — fix

id: BB-BUG-003
family: UIX
status: FIXED
source: PR #9 (`505e99b`, merged as `fb11fb5`)

## The change

MCP crate, `bb mcp` commands, HARNESS panel, View menu item and fixture removed.
Guard tests assert no MCP entry point in the CLI, the `bb` binary or any menu.

## Proof

Gate 270 passed / 0 failed; a real-window screenshot shows Explorer and editor
only, reviewed before merge.

## Still to do

The user-facing cause, a stale build, is BB-BLD-001 (not fixed yet).
