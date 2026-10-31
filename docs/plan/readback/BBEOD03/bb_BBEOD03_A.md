# BBEOD03-A readback, 2026-11-01 (backback_1)

## Setup

- Solutions: `fixtures/_Harness`, MbTx (`MbTx/MbTx.slnx`, 17 projects) and
  Apex (`Apex/Apex.slnx`, 27 projects), both read-only, restore off.
- App: release `bb probe` on branch `bbeod-03-intel`. Server:
  roslyn-language-server 5.13.0-1.26514.10. Machine: M4 Pro, load 4.3-7.2.
- Steps: `bb probe <file> <line:col> <feature> --repeat 10` (1 first + 9 warm).
- Write check: marker file, 16 probes + one `bb open`, then
  `find MbTx Apex -newer marker` gave 0 entries.

## Probe table (after the fixes)

MbTx file `MbAuto/MbDesktop/Grammars/_ApexManager.cs`; Apex file
`src/ApexTest/Reporting/ApexReportScope.cs`. Load = until all projects loaded.

| spot | feature | answer | load | first | warm p50 / p95 |
|---|---|---|---:|---:|---:|
| MbTx 29:22 | completion | 58 items (Page members) | 5.85 s | 113 ms | 3.4 / 4.7 ms |
| MbTx 29:22 | hover | `Task Page.NavigateAsync(...)` | 6.25 s | 237 ms | 1.0 / 2.2 ms |
| MbTx 21:27 | definition | `_ApexSharedBrowser.cs:8:23` | 6.60 s | 37 ms | 0.4 / 0.9 ms |
| MbTx 11:13 | definition | metadata `Page` (decompiled) | 6.41 s | 633 ms | 0.8 / 8.4 ms |
| MbTx 21:27 | references | 5 (grep: 5) | 6.56 s | 697 ms | 504 / 504 ms |
| MbTx 21:27 | rename | 4 files, 5 edits | 6.84 s | 367 ms | 41 / 57 ms |
| MbTx 29:22 | actions | 2 refactors | 6.75 s | 425 ms | 174 / 198 ms |
| MbTx file | diagnostics | 1 (IDE0005) | 6.72 s | 369 ms | 8.9 / 9.5 ms |
| Apex 27:29 | completion | 14 items (ReportCollector) | 6.77 s | 83 ms | 3.0 / 26 ms |
| Apex 24:9 | hover | `class NUnit.Framework.TestContext` | 6.67 s | 95 ms | 0.6 / 1.5 ms |
| Apex 14:13 | definition | `ReportCollector.cs:6:21` | 5.97 s | 20 ms | 0.4 / 0.9 ms |
| Apex 24:9 | definition | metadata `TestContext` | 6.46 s | 432 ms | 0.7 / 5.4 ms |
| Apex 31:30 | references | 3 (grep: 3) | 6.54 s | 563 ms | 503 / 505 ms |
| Apex 31:30 | rename | 2 files, 3 edits | 6.83 s | 239 ms | 14 / 43 ms |
| Apex 27:29 | actions | 2 refactors | 6.79 s | 189 ms | 74 / 105 ms |
| Apex file | diagnostics | 3 (IDE1006 x2, IDE0130) | 6.40 s | 99 ms | 8.8 / 9.4 ms |

`open` (didOpen + first diagnostics pull): MbTx 1.78-1.89 s, Apex 1.27-1.33 s.

## 1. Issue: the load signal came before the load

- Observed: on Apex, completion 0 items, hover none, definition 0, and a
  false IDE0005 on `using ApexReports;`. The run log ended after 1 project.
- Cause: Roslyn sends `projectInitializationComplete` after its first batch:
  Apex 1 of 27 (26 more 3.4 s later), MbTx 1 of 17, harness 1 of 4.
  BBEOD01's MbTx "loaded 4.94 s" was measured on this early signal; the
  full load is 6.48 s (`bb open`, this run).
- Failing test: `open_waits_until_every_project_of_the_solution_has_loaded`
  (log held 1 of 4 projects). Unit: `the_init_signal_after_the_first_batch_waits_for_every_project`.
- Fix: `wait_for_load` also waits until the `Completed (re)load of N project(s)`
  lines add up to the solution's project count; a timeout names the counts.
  It guards both callers (`bb open`, `Session::start`). Repeat: table above.

## 2. Issue: completion was null as the first request

- Observed (harness): completion after `greeter.` gave 0 items, 40 tries
  over 10 s. Any other request first (hover, definition, pull) fixed it.
- Failing test: `completion_after_an_incremental_edit_lists_hello`.
- Fix: `open` pulls diagnostics once and returns them (also what the editor
  shows first). A failed pull closes the document again.

## 3. Issue: references listed the declaration twice

- Failing test: `definition_references_and_hover_reach_across_projects` (6, 5 unique).
- Family "server repeats an answer": one guard (`model/unique.rs`) for
  definition, references and rename edits; a repeated rename edit would
  corrupt a file.

## 4. Issue: an edit inside a CRLF pair drifted the server text

- Observed: the sync property test failed; shrunk to a delete that starts
  between `\r` and `\n`. LSP cannot point there; Roslyn clamps to the line
  end, before `\r`, and deleted one char more than the mirror.
- Failing tests: `an_edit_starting_inside_a_crlf_pair_keeps_the_server_text_equal`
  (3 of 3), `an_edit_boundary_inside_a_crlf_pair_is_widened_to_the_whole_pair`.
- Fix: a boundary inside a CRLF moves to cover the pair, and the covered
  `\r`/`\n` goes back into the text, so the result text is the same.
  Repeat: 12 property runs (288 random cases) green.

## Open items

- references: a fixed ~500 ms floor on every warm request, all 3 solutions.
  A `partialResultToken` also streams at 504 ms; documentHighlight takes
  1-29 ms. Server-side batching; no client fix found.
- Roslyn pushes no `publishDiagnostics`; bb pulls (`textDocument/diagnostic`).
- After load, the session drops server log lines (the run log stops there).
- Roslyn's formatter fails (`-32000 Unexpected true ... CodeShapeAnalyzer.cs
  line 36`) on some U+0085/U+2028 trivia; the property witness compares it
  as an outcome.

## Lesson

A server "ready" signal is a claim, not evidence: count what it says it
loaded against what the solution declares.
