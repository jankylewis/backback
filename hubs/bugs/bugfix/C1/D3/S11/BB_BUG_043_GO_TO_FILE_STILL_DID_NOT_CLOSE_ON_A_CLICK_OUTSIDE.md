# BB-BUG-043 — fix

id: BB-BUG-043
family: BLD
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/status.rs` (`build`), `editor/view/pane/window_tests.rs`

## The change

No code change to the box was needed (BB-BUG-042 holds). The status bar
now names the build (`Ready · <commit>`, from the bundle's `BBCommit`),
so a stale install shows at a glance; the fix is installed with it.

## Proof

Window test `go_to_file_closes_on_a_press_anywhere_outside_it_with_two_panes`
(four places, split layout). `bb_build_state` in readback names the commit.
