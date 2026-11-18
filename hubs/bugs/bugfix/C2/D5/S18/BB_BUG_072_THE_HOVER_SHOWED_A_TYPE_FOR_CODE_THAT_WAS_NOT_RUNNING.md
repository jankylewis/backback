# BB-BUG-072 — fix

id: BB-BUG-072
family: DAP
status: FIXED
source: branch `fix/five-debug-window`; `view/bb_panel/stale.rs`, `view/inspect.rs`, `DebugState::started`

## The change

The session records when it began (its build starts then). A paused file is stale when its tab is dirty or saved after that; there a name the debugger cannot evaluate shows `no value: file changed since the debug build — restart to rebuild`.

## Lesson

A fallback that answers from a different source (the editor's text, not the running build) must say which source it is.

## Proof

`a_file_changed_since_the_debug_build_says_so` (window; mutations of the rule and of the hover branch both red); `a_file_is_stale_when_unsaved_or_saved_after_the_build_began`. Normal case: `set_next_moves_up_inside_an_async_spec_and_its_late_local_shows` evaluates `x` = 1 in a saved, built async spec.
