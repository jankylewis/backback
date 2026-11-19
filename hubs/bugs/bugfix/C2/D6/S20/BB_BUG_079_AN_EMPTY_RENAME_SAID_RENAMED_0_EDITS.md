# BB-BUG-079 — fix

id: BB-BUG-079
family: LSP
status: FIXED
source: branch `fix/real-tools`; `intel/glue/answers/mod.rs` (`Answer::Rename` with no change)

## The change

A rename answer with no change opens no preview and says `Nothing to
rename here`, as empty definition and code-action answers already do.

## Proof

`a_rename_with_no_edit_says_so_and_opens_no_preview` (window, F2 + keys,
fake server answers no edit): red before (a preview opened), green after.
The real-pyright window test renames once pyright answers.
