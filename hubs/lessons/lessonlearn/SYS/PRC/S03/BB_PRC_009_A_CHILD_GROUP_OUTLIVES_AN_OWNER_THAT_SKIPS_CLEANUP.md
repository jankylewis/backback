# BB-PRC-009 — lesson

id: BB-PRC-009
family: PRC
status: FIXED
source: BBEOD42 (headmate request 2026-11-20)

## Who believed what

`kill_on_drop`, `OwnedGroup`, the quit hook and the signal handler end
every child, so nothing can outlive backback.

## What changed the belief

Ten `dotnet` processes with ppid 1 for 16.5 h. Every one of those paths
is code the dying process must run; a SIGKILL, an abort or an exit runs
none. The first kill test passed on the old code because the servers end
on stdin EOF by themselves; only a child that does not watch backback (a
build step) showed the hole.

## Rule left behind

An owner that can die gets a watcher outside itself that the kernel
wakes (a pipe end), never cleanup code of its own. Prove it by killing a
real process holding a child that ignores its parent, not by a drop.
