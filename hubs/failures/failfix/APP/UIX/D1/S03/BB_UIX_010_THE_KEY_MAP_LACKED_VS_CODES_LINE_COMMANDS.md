# BB-UIX-010 — fix

id: BB-UIX-010
family: UIX
status: FIXED
source: branch `bb-bug-020-line-commands`

## The fix that holds

One text-model command per line command, one undo step each, mapped in
`document_command`: `delete_lines`, `insert_line_below`,
`insert_line_above`, `select_line` (Cmd+L). Tab's selection path became
`indent_lines`. A new line takes the file's own
line break (`enter`'s rule).

## The wrong fix

Arms for Cmd+[ and Cmd+] in the editor: the app binds them to Back and
Forward, which run first. The window test passed anyway (its rig has no
app bindings); the readback on the real app showed nothing changed.

## Done since

Move and copy lines (Alt+Up/Down, Shift+Alt+Up/Down): `move_lines` and
`copy_lines` in the text model, arms in `text_key` (BB-BUG-021). Cmd+[ /
Cmd+] still wait for the headmate's ruling; Back/Forward kept.
