# BBEOD03-DIAGVER readback, 2026-11-01 (backback_1)

- Branch `bbeod-03-diagver` from `3d83ae0`. Trigger: in BBEOD04-VERIFY one
  Roslyn worker test got an empty diagnostics batch after an edit.
- Family: **diagnostics not tied to the document version.** A list computed
  for one text is kept, returned or drawn for another text.

## Instances and fixes

| Instance | Fix |
|---|---|
| Pump stored every push, ignored `report.version` | `model::store`: each entry has its version; an older or unversioned report never replaces a newer one |
| `Session::diagnostics` returned `collected()`, not its answer | returns `DiagnosticsReport { version, list }` of this answer |
| A non-Full pull answer gave the old list | Unchanged keeps a list only of the same version; Partial, or Unchanged without it, is `NoReport` |
| Poisoned store lock gave `Vec::new()` | `IntelError::StorePoisoned` at every reader, `close` and the pump log |
| `Event::Diagnostics` had no version | carries `DocVersion` |
| Window drew any list it got | `model::shown::Diags`: each `Open` sent is v1, each `Change` the next; a list shows only on the version shown |
| Sibling: LSP request with a poisoned pending map waited for the deadline | fails at once with `Closed` |

Siblings seen, not fixed (debugger, outside this unit): in `bb_dap`,
`capabilities()` and `target()` turn a poisoned lock into `None`.

## Failing tests first

- Store, against the old blind insert: 3 of 6 failed (stale report kept,
  2 property tests). The poison test cannot pass on the old type.
- Window and model, against the old rule: 4 of 5 failed (old list on new
  text, stale report shown).
- `entry_of`, against the first fix: an Unchanged answer after an edit
  kept the old list (found by the live readback, below).

## GPUI readback (harness copy, `BB_KEYS`, debug build)

Keys: rename `actual` to `zactual` on line 14 (CS0103 at 15:21), then one
space at the start of line 15. `state` prints the version and squiggles.

| Step | Version | Squiggles on line 15 |
|---|---|---|
| loaded | 1 | 15:9, 15:29 |
| right after `z` | 2 | none (old list hidden) |
| after the pull | 2 | 15:9, 15:21, 15:29 |
| right after the space | 3 | none |
| after the pull | 3 | 15:10, 15:22, 15:30 (moved with the edit) |

The saved file matched: `string zactual`, line 15 one space deeper.

**Readback found a bug in the first fix.** It sent the last result id with
each pull. Roslyn answered "unchanged" after the edit, and backback put the
old list under the new version: squiggles one edit behind (15:9 after the
space). With `previous_result_id: None` the lists were right. Fix: never
send it; trust Unchanged only for a list of the same version.

## Gate and loop

- Gate 5 (`--all-features`, `BACKBACK_NETCOREDBG` set): fmt, clippy, 41
  suites 279 passed 0 failed 0 ignored, deny, `bb audit` clean, path guard.
- 20x `intel::bb_worker::tests` at 2 threads: 20/20, both before and after
  the Unchanged fix; every log kept in the seat scratch.
- Gate 4 failed once in `open_waits_until_every_project_of_the_solution_has_loaded`:
  `_Core.csproj` loaded twice (`(re)load of 1`, then `of 4`), sum 5 of 4.
  Not this unit; it is the next one.

## Lessons

1. **A result has the version of the input it was computed from.** Carry
   that version from producer to screen; drop or hide a mismatch.
2. **A server "unchanged" is relative to its own state, not your text.**
   Do not move an old list onto a newer version on its word.
3. **A poisoned lock is an error.** An empty default looks like a real
   answer ("no problems") and hides the fault.
4. **Readback beats the unit layer for server behaviour.** Every unit
   test was green while the live server put squiggles one edit behind.
