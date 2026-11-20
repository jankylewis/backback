# BB-PRC-009 — fix

id: BB-PRC-009
family: PRC
status: FIXED
source: branch `bbeod-42-no-orphan-servers`, `bb_core/process/src/orphan/`

## The fix that holds

A guard needs no code in backback to run. It is one `/bin/sh` per
backback, in its own process group, holding the read end of a pipe whose
only write end is backback's. Every registered group is told to it
(`+ <pgid> <lstart>`, `- <pgid>` when ended). However backback ends, the
kernel closes the pipe: the guard sends TERM to each group still watched
whose leader is the same process (`lstart` token) or gone with members
left, then KILL after 1 s. A record file per process in
`$BACKBACK_STATE_DIR/children/` is the backstop: a launch ends the groups
of records whose owner is dead, only when the token still matches, and
refuses a format it does not know. The debugger's `dotnet build` now runs
in an owned group, with `-nodeReuse:false -p:UseSharedCompilation=false`.

## The wrong fix

More cleanup in backback (an `atexit`, a bigger quit hook): a SIGKILL or
an abort runs none of it. An env tag on children: `ps` on this macOS shows
no other process's environment, so it cannot be read back.

## Proof

`no_orphan_servers` with `BACKBACK_REQUIRE_TOOLS=1`: 5 of 8 red at 5ad7d19
(build, debug, panic, unsaved-tabs + force quit, launch sweep), 9/9 green
(a 9th, SIGKILL during the debugger's build, kills mutant M10a); bb-process
guard, sweep and SIGKILLed-probe tests; 14/15 mutants killed. `evidence/BBEOD42_READY.md` in the fleet state.
