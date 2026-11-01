# BBEOD04-B Test Explorer readback, 2026-11-01

- Seat: backback_2; branch: `bbeod-04-gui` from `a10a5f3`.
- Input: scratch .NET 10 NUnit project with one passing and one failing
  `[Test]` method. No MbTx or Apex spec was run or listed in this unit.
- App: native macOS GPUI desktop, local scratch project and source file.

## Readback loop

1. Opened the scratch source, selected Tests, expanded its project. The
   initial native run kept the project spinner active until VSTest timed out.
   The same `dotnet vstest` command completed from the shell in about 0.5 s.
2. Moved discovery and execution to dedicated worker threads. The native
   script then reported `projects=1 discovered=2 loading=0` within 8 s.
   The process-level `bb_native_discovery` test exercises this served path;
   its pre-fix witness was `discovered=0 loading=1`.
3. In the native app, expanded namespace and class, ran the class, observed
   one passed and one failed spec, and selected the failed spec. The detail
   showed its failure message and stack; clicking the source place opened
   `OutcomeSpec.cs` at line 5. The editor showed `[Test]` gutter run icons.
4. Inspected [bb_FAILURE_JUMP.png](bb_FAILURE_JUMP.png): the served window shows
   the result counts, disabled Debug, failure detail, and source line.

## Tests and remaining work

- In-process GPUI input test covers open, lazy list, class run, outcomes,
  failure selection, and source navigation. Model tests cover rediscovery;
  tree-sitter tests cover attribute detection. A process-tree stop test
  verifies cancellation kills a spawned child.
- Debug is disabled with the tooltip “Available once the debugger lands” by
  user ruling. DAP attach wiring is open until BBEOD05 merges to master and
  a later integration unit owns it. BBEOD05 was not merged into this branch.
- The live MbTx/Apex listing path was not rerun in this GUI unit. BBEOD04-A
  recorded its read-only scratch listing separately. No live spec run.

## Lesson

An in-process GPUI test can miss a subprocess wait that hangs only in the
native app. Keep a native process-level guard for discovery.
