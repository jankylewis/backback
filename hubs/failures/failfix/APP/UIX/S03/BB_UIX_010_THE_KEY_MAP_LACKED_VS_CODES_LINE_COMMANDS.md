# BB-UIX-010 — fix

id: BB-UIX-010
family: UIX
status: OPEN
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

## Still to do

Move and copy lines (Alt+Up/Down, Shift+Alt+Up/Down) and a ruling on
Cmd+[ / Cmd+]: BB-BUG-021.
