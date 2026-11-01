# BBEOD03-FIX readback, 2026-11-01

## Discovery writes beside the test assembly

- Family: a tool invoked for a read-only answer writes beside a user's build.
- Repro: `bb test list` on a built `_Harness` copy changed
  `_NUnitApex/bin/Debug/net10.0/nunit_random_seed.tmp` after a marker.
- Failing-first test: `list_leaves_the_built_solution_tree_unchanged` failed
  with that exact new file. It now passes for VSTest, MTP, and Apex.
- Guard: discovery runs from a temp shadow of the build output. The copy
  admits runtime assets only. `shadow_copies_only_runtime_assets` checks that
  an unrelated JSON file is not copied.
- Real readback: `bb test list` on Apex.Cli.UnitTests (Debug, VSTest) exited 0;
  a full Apex tree scan found 0 files newer than the command start.

| Command or path | Write state | Witness |
|---|---|---|
| `bb open` default | Traceless | Existing full-tree snapshot test passed. |
| `bb open --restore` | May write restore output | Explicit restore test passed. |
| `bb probe` | Traceless | New full-tree snapshot test passed. |
| GUI Roslyn open and sync | Traceless | Real worker full-tree snapshot test passed. |
| `bb test list` | Traceless | CLI marker test passed for all three backends. |
| `bb test run`, no build, VSTest | Runner output in temp | CLI marker test passed on NUnit Smoke. |
| `bb test run` with build enabled | May write build output | Existing Apex fixture enables build. |
| GUI test explorer, `bb debug`, `bb mcp` | No path exists | Source and CLI command scan found none. |

The GUI test explorer has no served path in this branch. Its future caller
must use the same testkit list function and add a GUI trace test.

## Roslyn death leaves Ready

- Family: EOF closes transport waiters but the UI keeps a stale Ready state.
- Failing-first instrument test named `a_dead_server_stops_within_one_second_`
  `and_restart_answers_again` failed to compile before the stopped event.
- Guard: the transport publishes EOF; the worker emits Stopped; the UI clears
  pending questions and shows `C# server stopped — Restart`.
- The status bar has a Restart button. The C# menu and Cmd-Shift-R run the
  same action. Restart relaunches the solution and replays current tab text.
- Real child kill test: Stopped arrived within one second. The new session
  opened the document and answered completion. A GPUI window test checked
  the stopped label, pending-question clear, and unsaved text replay.
