# BB-BUG-019 — fix

id: BB-BUG-019
family: LSP
status: FIXED
source: `bb_crates/shell/desktop/src/intel/bb_worker/bb_loop/mod.rs` (`mark_open`), `edge/intel/src/bb_session.rs` (`refresh_signal`), `edge/bb_lsp/src/roslyn/mod.rs` (`initialize_params`)

## The change

A change marks every open document for the pull after the quiet gap, not
only the one it edited. bb also declares `workspace/diagnostic/refresh`
support; a refresh request or a project (re)load line marks every open
document too, so a restore outside the editor re-pulls them as well.

## Proof

Red first: `an_edit_in_one_file_refreshes_the_diagnostics_of_the_other_open_files` (server thread, real Roslyn) renames `Greeter.Hello`
and waits for the missing-member error in the spec's own list, then for it
to clear on undo, with no edit of the spec. With master's server loop:
fails after 94.6 s (`the spec never got a list where the call to the renamed Hello fails within 90 s`). With the fix: passes in 5.2 s. Readback: a rename on a restored copy leaves `problems=1` and a stale squiggle in MathOpsTests.cs on the installed `c145669`, `problems=0` on the branch build `6d945f5`; a restore outside the app keeps 10 NUnit errors before, 10 -> 0 after (`docs/plan/readback/V2/SWEEP01/BUG_019_READBACK.md`).
