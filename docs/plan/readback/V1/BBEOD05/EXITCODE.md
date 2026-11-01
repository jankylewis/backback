# BBEOD05-EXITCODE launch exit code readback (2026-11-01)

- Seat: `backback_1`; branch: `bbeod-05-ship`.
- netcoredbg: local clone `backback_wt/netcoredbg`, branch `local-setnext`,
  commit `9731de2` on top of Set Next `9393231`. Local only, not pushed.
- Adapters: stock `3.2.0-1` (`9744e1f`, Release) and the local patched build.
  Both print the same `--version` (`9744e1f`), so backback detects the patch
  by a capability, not by version.

## Root cause

`libdbgshim.dylib` imports `_waitpid (from libSystem)` (`nm -m`). macOS
binds symbols per library (two-level namespace), so the `waitpid()` hook
that netcoredbg exports never sees the child status. The default `0` stays.

## Issues, failing tests, fixes

| Bug | Failing test (before fix) | Fix |
|---|---|---|
| Launch `Main` returns 3, `exited` says 0 | `VSCodeTestExitCodeReturn` (stock and patched): `exitCode 0` | kqueue `EVFILT_PROC` `NOTE_EXIT\|NOTE_EXITSTATUS` on the launched child |
| Upstream `VSCodeTestExitCode` (`_exit(3)`) | failed at `WasExit(3)` on both | same fix; now passes |
| `MITestExitCode` | failed on stock | same fix; now passes |
| Sibling: SIGKILL death shown as `code 1` | `launched_signal_death_...`: `Verified(1)` | adapter adds `netcoredbgSignal`; backback maps it to `Signaled` |
| backback: launch code always `Unverified` | `launched_exit_code_...`: `Unverified(3)` with patched | `launch_exit_truth` is `Verified` only with capability `netcoredbgLaunchExitCodeIsProcessStatus` |

Attach is not covered: `NOTE_EXITSTATUS` is for a child only. Attach to a
process backback started keeps the owned-status path (`exit_truth`).

## User readback (`bb debug`, `Main` returns 3, direct run status 3)

| adapter | `exit code` line | `exited` line |
|---|---|---|
| stock | `adapter word` | `code 0 (adapter-reported, unverified; ...)` |
| patched | `process status` | `code 3` |

Signal case (`Process.Kill()`, direct run status 137): stock shows
`code 0 (adapter-reported, unverified; ...)`; patched first showed `code 1`
(wrong), after the fix `ended by a signal`.

## Lessons (bug families)

1. **Interposition is not portable.** An exported libc hook works on Linux
   (flat ELF lookup) and silently does nothing on macOS. Check with
   `nm -m` which library a symbol binds to before trusting a hook.
2. **Confident default for an unknown value** (family from FIX.md). A
   fallback value (`0`, `EXIT_FAILURE`) must carry its source. Guard: the
   adapter states what it knows (capability, `netcoredbgSignal`), and
   `launch_exit_truth` trusts nothing else.
3. **Same version, different build.** A version string can hide a local
   patch. Detect behaviour by a capability the patch adds.
