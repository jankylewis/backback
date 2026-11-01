# BB-TG-001 — netcoredbg has no `runInTerminal`

id: BB-TG-001
for: none
status: OBSERVED
date: 2026-11-01
source: upstream netcoredbg `src/protocols/vscodeprotocol.cpp` lines 594-627

The stock adapter never sends the DAP `runInTerminal` request, so backback
cannot own the debuggee process or read its real exit code. `for: none`: no
failure yet, a limit found by reading the adapter.

Workaround: show the exit code as "adapter-reported, unverified". Patches stay
local until the headmate rules on upstream.
