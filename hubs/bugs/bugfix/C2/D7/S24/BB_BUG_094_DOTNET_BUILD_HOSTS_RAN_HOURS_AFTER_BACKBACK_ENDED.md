# BB-BUG-094 — fix

id: BB-BUG-094
family: PRC
status: FIXED
source: branch `bbeod-42-no-orphan-servers`

## The change

A guard process per backback ends every owned group when backback ends in
any way; a launch ends the groups a dead backback recorded, and only those
(BB-PRC-009). The debugger's `dotnet build` runs in an owned group with no
node reuse and no shared compiler.

## Proof

`no_orphan_servers`: SIGKILL, SIGTERM, quit, panic and a force quit at the
unsaved-tabs prompt, with Roslyn, its `BuildHost`, rust-analyzer, pyright,
netcoredbg and a build running: nothing left after 5 s; 5 red at 5ad7d19.
See `evidence/BBEOD42_READY.md` in the fleet state.
