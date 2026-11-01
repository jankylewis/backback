# BBEOD06-A MCP readback, 2026-11-01

> Removed by headmate ruling 2026-11-01 (BBEOD06-REMOVE): backback has no MCP server ability. History only.

- Seat `backback_2`; branch `bbeod-06-mcp`; source `978f75f`.
- Fixture: `_Harness/McpServer` only. No real fleet MCP server was started.
- `bb mcp list fixtures/_Harness/Harness.slnx` listed `_McpServer.csproj` with `dotnet run --project`.
- `bb mcp tools fixtures/_Harness/McpServer/_McpServer.csproj` listed `Greet` and `Hold`.
- `bb mcp call ... Greet '{"name":"Vinh"}'` returned `hi Vinh`.
- `bb mcp call ... Greet '{}'` returned a typed tool error (`isError`).
- Killing `Hold` mid-call failed the waiter. The process group had no remaining member by `pgrep -g`.
- Graceful `stop` returned success. The process group had no remaining member by `pgrep -g`.
- `restart` stopped the old group, started a new fixture process, answered ping, and left no group after stop.
- `bb mcp log` printed the fixture's startup stderr with a timestamp and exited 0 after SIGINT.

## Read-only solution discovery

| Solution | Observed MCP projects |
|---|---|
| `MbTx/MbTx.slnx` | None in the solution listing |
| `dna-nemocatalog-tx/NmcTx/_NmcTx.slnx` | `NmcAx/mcps/_NmcMcp/NmcMcp.csproj` |
| `msi/.../_SiTestLegos/_SiTestLegos.slnx` | `SiAxToolhub/_SiMcp/_SiMcp.csproj` |

Each scan exited 0. Discovery read project files only. The real MCP projects were never launched.

## Lesson

The MCP fixture had a tool class but no executable entry point. A real CLI call exposed this gap.
Keep a served-surface fixture call in the gate so the project must start and answer over stdio.

The log stream initially lost stderr emitted before the handshake completed.
Family: late subscribers miss startup events. A receiver now exists before spawning the reader.
The fixture emits `fixture boot`; its regression test failed before the fix and passes now.
