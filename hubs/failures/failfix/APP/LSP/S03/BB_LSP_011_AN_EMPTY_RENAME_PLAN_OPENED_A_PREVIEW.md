# BB-LSP-011 — fix

id: BB-LSP-011
family: LSP
status: FIXED
source: `Answer::Rename(changes) if changes.is_empty()` arm

## The fix that holds

No preview for an empty plan; the status says `Nothing to rename here`.

## Proof

`a_rename_with_no_edit_says_so_and_opens_no_preview` red before, green
after; mutation (arm disabled) red again.

## The wrong fix

Waiting in the window test until pyright has indexed: the user still
gets the empty preview.
