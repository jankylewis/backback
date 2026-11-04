# BB-BUG-041 — fix

id: BB-BUG-041
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/bb_tabs/mod.rs` (`pending_close`, `save_close`, `discard_close`), `bar/mod.rs` (`next_close`)

## The change

`pending_close` holds the file's path; Save and Discard find its tab when
pressed. The family check: every other index kept across a change
(`active`, `other_active`) is already re-found by path (`keep.rs`).

## Proof

Unit test `the_save_prompt_keeps_its_file_while_tabs_move`: red before
(c.cs closed, a.cs kept), green after. The readback state line names the
asked file (`asking=`).
