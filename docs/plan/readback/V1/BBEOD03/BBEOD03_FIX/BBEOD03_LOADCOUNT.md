# BBEOD03-LOADCOUNT readback, 2026-11-01 (backback_1)

- Branch `bbeod-03-diagver` (on `52386aa`). Trigger: bus 066, Roslyn
  loaded `_Core.csproj` twice (`(re)load of 1`, then `of 4`): the sum was
  5 of 4, and readiness ended with one project still unloaded.
- Scope add (backback_2): `bb --test bb_roslyn` failed on master. The probe
  needs ~42 s under load ~10, and the test helper kills it at 30 s. The
  rename probe hit the same wall in the BBEOD06 re-verify.

## Family 1: load state inferred by summing batch lines

Roslyn writes one line per project (`Successfully completed load of <p>`,
`Error while loading <p>: <why>`) and one summary per batch. A batch can
reload a loaded project, so the sum overshoots.

Guard: `bb_lsp::roslyn::LoadState` holds the set of distinct project paths
(canonical, else `.`/`..` folded). Batch lines are never counted. A failed
project settles the wait and is reported. The timeout names the missing
projects.

| Instance | Fix |
|---|---|
| `wait_for_load` summed `batch_projects` | waits on `LoadState::settled`; returns `Loaded { took, failed }` |
| desktop `ProgressLog` summed batches | one progress event per new distinct project |
| bb_cli e2e summed batch lines | counts distinct loaded paths |
| failed projects invisible | `bb open`/`bb probe` print `failed    :`; `Event::Ready` carries them; status bar names them |

`batch_projects` is removed, so no new caller can sum batches.

## Family 2: fixed wall around Roslyn readiness

| Instance | Before | After |
|---|---|---|
| bb_cli `common::bb` (open, probe) | 30 s for the whole run | waits for the `loaded    :` line (330 s, past bb's own 300 s), then 120 s |
| bb_cli other commands | 30 s | 30 s (`Plan::Flat`) |
| intel tests load / request | 120 s / 30 s | 300 s / 60 s (bb's own values) |
| desktop worker `wait_ready` | 180 s per event, no message | 300 s for the whole load, names the progress so far |

A stall is the typed `Stalled { pending: Ready | Exit, after, out, err }`.

## Failing tests first

- `load_tests.rs`: the bus 066 sequence with batch counts 1 + 4 = 5. Red
  first as a build failure (new API). The old sum (5 >= 4) passes this
  case, by reading the code; it was not run on the old code. The new state
  times out and names `_McpServer.csproj`. Also tested: a project that fails, a retry that
  then loads, and two spellings of one path.
- `bb_watch.rs`: a fake slow load (2 s before `loaded    :`, 1 s request
  budget). On the flat wall, 2 of 4 failed ("still waiting for the exit
  after 1s"). With the readiness wait, 4 of 4 passed.
- status bar: `status_label` and the window test were red (no function)
  before the wiring.

## Readback

- MbTx read-only `bb open`: 17 of 17 projects (`MbTx.slnx` declares 17;
  the brief said 11). 17 distinct load lines = 17 lines = batch sum 17;
  0 failures; loaded in 9.48 s. `find MbTx -newer marker`: 0 entries.
- 20x `open_waits_until_every_project_of_the_solution_has_loaded`: 20/20,
  three runs (before scope add, after it, final tree).
- 20x `intel::bb_worker::tests` at 2 threads: 20/20, three runs.
- Gate (`--all-features`, `BACKBACK_NETCOREDBG` set, 2 threads): fmt,
  clippy, 41 suites 289 passed 0 failed 0 ignored, deny, `bb audit` clean,
  path guard 0 hits. The first try had a wrong netcoredbg path (Set Next
  test: NotFound); the rerun with the right path is the count above.
- Proptest: any sequence of load, failure and batch lines gives the same
  state as a set model. Mutant check (drop the failure-after-load guard):
  the property failed. Not run: `cargo mutants`, kani (not installed).

## Lessons

1. **Count identities, not events.** A summary line (batch, page, retry)
   can repeat an item. Keep the set of what it names.
2. **Wait on the event, bound it with a deadline.** A flat wall measures the
   machine load, not the code. When it fires, name what was pending.
3. **A failure is an answer.** A project that fails to load ends the wait
   and is shown. Waiting longer only hides it.
