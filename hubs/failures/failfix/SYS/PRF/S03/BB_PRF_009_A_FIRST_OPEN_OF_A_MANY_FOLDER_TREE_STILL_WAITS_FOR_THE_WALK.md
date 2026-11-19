# BB-PRF-009 — fix

id: BB-PRF-009
family: PRF
status: FIXED
source: branch `perf/first-frame-no-wait` (BBEOD38)
source: `shell/desktop/src/app/bb_launch/open/langs/bb_early.rs` `resolve`, `when_found`
source: `shell/desktop/src/app/bb_launch/bb_window.rs` `languages_found`
source: `shell/desktop/src/editor/bb_tabs/tab/mod.rs` `Tab::recolour`

## The fix that holds

`resolve` returns `Resolved::Now` (a walk that already ended, or any
open that is not a launch: the Start Page with ticks, File > Open) or
`Resolved::Later` (the launch's walk still runs). With `Later` the window
opens at once as `WindowLangs::detecting()`: no language active, so no
tool starts, no Debug or C# menu, files open as plain text; the status
bar says "Detecting languages…" and F5 says "Debug: detecting languages;
try again in a moment". `when_found` polls the walk's answer (a oneshot
filled by the `bb-detect` thread) every 4 ms on a gpui timer; the answer
goes through the same `finish` as before (a complete walk with no saved
choice is saved), then `languages_found` sets the menus (window in
front), starts the servers and Cargo rows, recolours every open tab and
sends every open tab to the servers that started (`resync_open`). A tab
still loading recolours when it lands (`finish_open`); a tab mid-parse
recolours when its parse ends (`LiveTree::parsing`), so an old-grammar
parse never lands on a new tree.

## Wrong fix not taken

A parallel walk alone (68 ms on dna-ax) still exceeds the ~56 ms start
(ruling). Awaiting the oneshot directly: gpui's test scheduler refuses a
wake from a plain thread, so the tests could not run the app's path.

## Proof

Window tests `open/tests/first_frame.rs` with a slow fake walk (held
until released, 3 s at most): the window shows before the walk ends and
takes C#+Python after (menus C# and Debug appear, Roslyn starts); a
`src/main.rs` argument opens Plain then Rust with rust-analyzer started;
F5 while detecting says so; the Start Page path with ticks knows its
languages at once and walks on the UI thread as before.
Red: with `resolve` waiting for the walk (BBEOD36 behaviour) 3 FAILED,
"the window waited for the walk" / F5 "no console (Exe) project".
Mutations, each FAILED then restored (cmp OK): no recolour; no menus at
the answer; no F5 handler while detecting; no tool starts at the answer.
Not covered by a test: `resync_open` (window tests start no server).

dna-ax first open, medians of 10: base 184, BBEOD36 341, fix 189 ms
(+2.7 %); MbTx, 50,000 `.cs` and a saved choice within 2.5 %. Record:
`docs/plan/bench/bb_GENERAL/RERUNS/BBEOD38_FIRST_FRAME_NO_WAIT.md`.
