# BBEOD03-B readback, 2026-11-01 (backback_1)

## Setup

- App: release `backback` on `bbeod-03-gui`; server roslyn-language-server
  5.13, restore off. Keys and mouse go through the window's own dispatch
  (`BB_KEYS` with `wait:`, `goto:`, `hover:`, `mark:`, `state` steps);
  `screencapture -l` at each mark. No Accessibility trust is needed.
- Harness: a writable copy of `fixtures/_Harness`. MbTx and Apex: the real
  solutions with `--no-write`. Write check: marker file before the run, then
  `find MbTx|Apex -newer marker -type f` gave 0 files for both.

## Feature table (last run of each)

| feature | harness | MbTx | Apex |
| --- | --- | --- | --- |
| load to Ready | 2.6 s | 5.4 s | 7.0 s |
| completion | 5 items after `.` | Page members | ReportCollector members |
| hover | `Greeter.Hello(string name)` | `Page.NavigateAsync(...)` | `NUnit...TestContext` |
| F12 | Greeter.cs:3 | _ApexSharedBrowser.cs:8 | ReportCollector.cs:6 |
| F12 metadata | String.cs read-only | Page.cs read-only | TestContext.cs read-only |
| Shift+F12 | 5 | 5 | 3 |
| F2 plan | 3 files, 5 edits, applied | 4 files, 5 edits, refused | 2 files, 3 edits, refused |
| Cmd+. | 2 actions | 2 actions | 2 actions |
| Problems | 13 | 1 | 3 |

Counts match the BBEOD03-A probes on the same spots. The harness rename
saved the one closed file (GreetTool.cs) and left the two open tabs dirty.

## Latency (stage trace, step 3 of the brief)

| run | queued | server | hop to UI | frames to paint | answer to paint | key UI p99 / max |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| harness | 0.08 ms | 83 ms | 0.02 ms | 0 | 13.9 ms | 3.9 / 65.9 ms |
| MbTx | 256 ms | 109 ms | 0.02 ms | 0 | 19.8 ms | 1.9 / 2.0 ms |
| Apex | 0.02 ms | 100 ms | 0.03 ms | 0 | 13.1 ms | 2.1 / 2.9 ms |

- The popup drew in the first frame after the answer in every run.
- MbTx queued 256 ms: the first question waited behind the didOpen pull.
- The 65.9 ms key is only in the harness run, the only run that applied a
  rename (load, parse and save of a closed file on the UI thread). Not
  attributed per key.

## Issues found live, failing test first, fixed, steps repeated

1. Status fell back from Ready to "Loading 4/4 projects" after F12 into
   metadata: Roslyn logs a project batch line for the decompiled document,
   and every later question was refused. Family: a late event overrides a
   newer state. Tests: `a_late_batch_line_never_turns_ready_or_failed_back_into_loading`,
   `a_progress_event_after_ready_keeps_ready_and_questions_still_go`.
   Guard: `LoadStatus::progress` moves only a load in progress. Sibling:
   a hover answer for a char the mouse left drew a stale tooltip; test
   `a_hover_answer_for_a_char_the_mouse_left_shows_nothing`, guard: hover
   applies only while the mouse rests on the asked char.
2. Long Problems rows widened the editor column and squeezed the Explorer
   (editor at x=107, Explorer 300). Test
   `long_problem_rows_never_squeeze_the_explorer`. Then the rows wrapped
   inside 20 px and overlapped. Family: long content in a fixed layout.
   Guard: one `one_line` row helper (no wrap, clip, ellipsis) for every list
   row, a source test that finds no other fixed-height row (4 at HEAD), the
   Explorer does not shrink, the editor column clips.
3. Problems listed 275 hints of the decompiled String.cs. Test
   `problems_list_only_files_inside_the_workspace`; squiggles stay.
4. A file argument rooted the workspace at the file's folder, which has no
   solution, so intel stayed off. Test
   `a_file_argument_roots_at_the_nearest_folder_with_a_solution`.

## Open

- Squiggles keep their old columns after an edit until the next pull
  (250 ms after the queue is quiet).
- Near the window bottom the completion list opens upward over the cursor
  line; a panel can cover the cursor line (no scroll to keep it visible).
- The server thread is serial: a slow answer (references ~500 ms) delays the
  next question.
- Code actions are listed only, as the brief asks; no apply.

## Lesson

A status that one stream can set must say which states it may leave.
Check every long list row with long text before a screenshot calls it done.
