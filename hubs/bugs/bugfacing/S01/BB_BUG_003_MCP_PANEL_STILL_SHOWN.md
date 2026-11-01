# BB-BUG-003 — MCP HARNESS panel still shown after its removal

id: BB-BUG-003
family: UIX
status: FIXED
date: 2026-11-01
signature: HARNESS Select an MCP server
source: PR #9 (`fb11fb5`): MCP crate, panel and menu item removed; guard tests
observed: headmate screenshot 2026-11-01 showed the HARNESS panel after #9 merged
cause: BB-BLD-001
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open the backback window the headmate already had running.

Expected: no MCP panel (removed in #9).
Actual: HARNESS panel, "Select an MCP server".

## Note

The code was already fixed. The running window was an older build; see
BB-BLD-001 for why that happens and its fix (`install.sh`).
