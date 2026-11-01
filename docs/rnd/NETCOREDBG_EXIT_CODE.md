# netcoredbg exit code on macOS: upstream issue draft

Status: draft text only, not filed. Observed 2026-11-01 by backback
(BBEOD05-FIX B3). Root cause and a local fix: BBEOD05-EXITCODE.

## Title

`exited` event reports `exitCode: 0` on macOS when the program returns non-zero

## Environment

- netcoredbg `3.2.0-1`, source `9744e1f` (stock Release build), macOS arm64, .NET 10.
- Also seen with a local Debug build of the same tree plus a Set Next patch (`9393231`).
- Client: backback DAP client, `--interpreter=vscode`.

## Steps

1. Build a console app whose `Main` returns `3`.
2. Run it directly: the shell shows exit status `3`.
3. DAP `launch` it, then `configurationDone`, and let it run to the end.
4. Read the `exited` event.

Upstream's own `VSCodeTestExitCode` (`_exit(3)`) and `MITestExitCode` also
fail on macOS with the stock build.

## Expected

`exited` carries `exitCode: 3`, as the DAP specification asks: "The exit
code returned from the debuggee."

## Observed

| case | real status | `exited.exitCode` |
| --- | --- | --- |
| launch, `Main` returns 3 | 3 | 0 |
| attach, `Main` returns 5 | 5 | 0 |
| launch, SIGKILL | 137 (signal 9) | 0 |

## Root cause (verified)

- `ManagedCallback::ExitProcess` on `FEATURE_PAL` reads
  `GetWaitpid().GetExitCode()`; the default is `0`.
- Only the exported `waitpid()` hook (`src/debugger/waitpid.cpp`) changes it.
- `nm -m bin/libdbgshim.dylib` shows `_waitpid (from libSystem)`. macOS binds
  each import to a named library (two-level namespace), so the PAL in
  libdbgshim never calls the hook. The default `0` is sent.
- Attach: netcoredbg is not the parent, so no wait status exists for it.

## Local fix (commit `9731de2`, not published)

- On macOS, `SetupTrackingPID` adds a kqueue `EVFILT_PROC` watch with
  `NOTE_EXIT | NOTE_EXITSTATUS`. For a child, the kernel gives the wait
  status without reaping it. `GetExitCode()` waits up to 3 s for it.
- A signal death keeps `exitCode: 1` (upstream convention) and adds the
  non-standard field `netcoredbgSignal`.
- Non-standard capability `netcoredbgLaunchExitCodeIsProcessStatus: true`
  (macOS) tells a client the launch code is real.
- Tests: `VSCodeTestExitCodeReturn` (exitCode 3) and `VSCodeTestExitSignal`
  (exitCode 1, signal 9). Both fail on stock and pass on the patched build.
- For upstream, the capability and the signal field are open questions: a
  plain fix needs neither, but a client cannot then tell old from new.

## What backback does

- Owned process (backback started it, then attached): the real OS status wins;
  `bb_dap::exit_truth` returns `Corrected { process, adapter }` on a mismatch.
- Launched, adapter with the capability: `launch_exit_truth` gives
  `Verified(code)`, or `Signaled` when `netcoredbgSignal` is present.
- Launched, stock adapter: `bb debug` prints
  `code 0 (adapter-reported, unverified; ...)`, never a bare `0`.
