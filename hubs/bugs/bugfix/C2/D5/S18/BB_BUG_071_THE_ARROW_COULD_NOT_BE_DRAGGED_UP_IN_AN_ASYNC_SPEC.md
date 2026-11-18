# BB-BUG-071 — fix

id: BB-BUG-071
family: DAP
status: OBSERVED
source: branch `fix/five-debug-window`; `bb_core/syntax/src/lib.rs` (`statement_line`), `editor/bb_tabs/tab/mod.rs` (`set_next_place`), `view/notes.rs` (refusal)

## The change

The drop and Ctrl+Shift+F10 share one rule: the line means the innermost C# statement covering it (tree-sitter), so a later line of a statement sends its first line. A refusal is said in the status bar and kept in the Debug output.

## Lesson

Prove each layer of a "does nothing" report in turn (adapter, worker, window) before changing any; here the adapter and the worker were right, and the gap was the line sent.

## Proof

`a_line_inside_a_statement_means_that_statement`; `the_arrow_dropped_inside_a_multi_line_await_runs_that_statement` (window, mouse drag; mutations of the mapping and of the output echo both red); `set_next_moves_up_inside_an_async_spec_and_its_late_local_shows` (real netcoredbg, NUnit async spec, two moves up, each line ran next). Status OBSERVED until the headmate drags in the installed app.
