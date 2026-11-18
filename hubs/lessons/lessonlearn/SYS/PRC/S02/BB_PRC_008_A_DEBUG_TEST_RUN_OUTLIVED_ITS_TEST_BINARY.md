# BB-PRC-008 — lesson

id: BB-PRC-008
family: PRC
status: FIXED
source: BBEOD25 session, matemaster interrupt

## Who believed what

`OwnedGroup` ends its group on drop, so no debug run can leak.

## What changed the belief

An orphaned `dotnet test` after a green test: a drop never runs on a
thread the process exit kills.

## Rule left behind

A detached thread that owns a process is joined (or its channel drained
to closed) before its owner returns, in tests too. Pipe a test run to a
file, not to `tail`: an orphan holds a pipe open.
