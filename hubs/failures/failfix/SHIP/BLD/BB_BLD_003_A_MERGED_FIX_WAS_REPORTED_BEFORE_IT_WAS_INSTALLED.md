# BB-BLD-003 — fix

id: BB-BLD-003
family: BLD
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/status.rs` (`build`, `commit_in`)

## The fix that holds

The status bar names the build: `Ready · dd0ccf8` (`dev` outside a
bundle). Before I say a fix is in the app, I check `BBCommit` holds it.

## The wrong fix

Installing while the app is open: the install script refuses, and an
open app keeps running the old binary anyway.
