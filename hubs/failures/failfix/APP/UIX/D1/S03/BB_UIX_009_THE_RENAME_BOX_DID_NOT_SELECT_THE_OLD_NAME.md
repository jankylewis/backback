# BB-UIX-009 — fix

id: BB-UIX-009
family: UIX
status: FIXED
source: branch `bb-bug-018-rename-select`

## The fix that holds

`RenameInput::fresh`: the prefilled name opens selected (drawn as one), the
first typed char replaces it, Backspace clears it; afterwards the box edits
char by char as before.

## The wrong fix

Opening the box empty (the user loses the name to edit a suffix), or
selecting only until the first key without drawing it (the user cannot
see what Enter will send).

## Proof

`the_box_opens_with_the_old_name_selected_so_typing_replaces_it`; the rig's
`f2 backspace T o t a l enter` still gives `Total`; sweep 3 H6 on this
build.
