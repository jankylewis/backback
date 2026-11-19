# BB-PRF-007 — fix

id: BB-PRF-007
family: PRF
status: FIXED
source: branch `perf/lang-no-regression` (BBEOD36)
source: `shell/desktop/src/app/bb_launch/open/langs.rs` `walk_early`, `early_walk`
source: `shell/desktop/src/app/bb_launch/mod.rs` `launch`

## The fix that holds

A launch with a folder argument starts the same walk on its own thread
(`bb-detect`) right after the arguments are read, before the app and the
window start. The saved choice is read on the calling thread first, as
`resolve` reads it. `resolve` takes that walk's result when it was for the
same root and ticks (`early_walk`, a join), else walks as before. Saving
the choice stays on the UI thread. Same walk, same budget, same result;
it now overlaps the app's own start (about 130 ms from exec to
`open_workspace` on this machine).

## Wrong fix not taken

Opening every window as C# first and changing the languages when the walk
ends: tools would start and stop, and the status bar would change under
the user. Skipping the walk for big folders: a different answer.

## Proof

Red first: `a_launch_walks_its_folder_once_off_the_ui_thread_and_the_window_shows_what_it_found`
(`open/tests/early_walk.rs`) with `walk_early` a no-op: FAILED, "the
folder was walked on the UI thread" (ThreadId(2) both sides). Fixed:
GREEN 3/3. Mutation (window ignores the early walk): FAILED, "one walk of
the folder: [ThreadId(3), ThreadId(2)]".

First frame, medians of 5, release, `aa1d223` / `19dd6d3` / fix:
MbTx 204 / 328 / 213 ms; 50,000 `.cs` files 191 / 274 / 193 ms; the same
through `open` 244 / 322 / 242 ms. Record:
`docs/plan/bench/bb_GENERAL/RERUNS/BBEOD36_LANG_NO_REGRESSION.md`.

## Limit left

A folder whose walk takes longer than the app's start (dna-ax-sized trees
walk 230-290 ms warm) still waits for the rest of the walk at the window.
