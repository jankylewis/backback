# netcoredbg exit code on macOS: upstream issue draft

Status: draft text only, not filed. Observed 2026-11-01 by backback (BBEOD05-FIX B3).

## Title

`exited` event reports `exitCode: 0` on macOS when the program returns non-zero

## Environment

- netcoredbg `3.2.0-1`, source `9744e1f` (stock Release build), macOS arm64, .NET 10.
- Also seen with a local Debug build of the same tree plus a Set Next patch (`9393231`).
- Client: backback DAP client, `--interpreter=vscode`.

## Steps

1. Build a console app whose `Main` returns `3` (or `5`).
2. Run it directly: the shell shows exit status `3`.
3. DAP `launch` it, then `configurationDone`, and let it run to the end.
4. Read the `exited` event.

Second case: start the app yourself, DAP `attach` to its pid, then let it exit.

## Expected

`exited` carries `exitCode: 3` (launch) and the real code (attach), as the
DAP specification asks: "The exit code returned from the debuggee."

## Observed

| case | real status | `exited.exitCode` |
| --- | --- | --- |
| launch, `Main` returns 3 | 3 | 0 |
| attach, `Main` returns 5 | 5 | 0 |

Backback's live test `owned_attach_exit_code_is_the_process_status` logs:
`adapter_exit_code=Some(0) process_exit_code=Some(5)`.

## Source reading (not a verified root cause)

- `ManagedCallback::ExitProcess` on `FEATURE_PAL` takes the code from
  `GetWaitpid().GetExitCode()` (`src/debugger/managedcallback.cpp`).
- `waitpid_t::SetupTrackingPID` sets the default `exitCode = 0`
  ("same behaviour as CoreCLR have"), and only the `waitpid()` hook in
  `src/debugger/waitpid.cpp` changes it.
- Attach: netcoredbg is not the parent of the target, so no `waitpid()` can see
  its status; the default `0` is always sent.
- Launch (hypothesis): on macOS the exported `waitpid` hook may not intercept
  the PAL's own `waitpid` calls inside the runtime libraries (two-level
  namespace), so the default `0` stays. Not confirmed with a debugger trace.

## Suggested fix

- When the status is unknown, omit `exitCode` or send an explicit unknown,
  instead of a confident `0`.
- On macOS, take the code from the runtime (for example the latched exit code)
  or from a real `waitpid` on the launched child.

## What backback does meanwhile

- Owned process (backback started it, then attached): the real OS status wins;
  `bb_dap::exit_truth` returns `Corrected { process, adapter }` on a mismatch.
- Launched by the adapter: backback cannot read the status, so `bb debug`
  prints `code 0 (adapter-reported, unverified; ...)`, never a bare `0`.
