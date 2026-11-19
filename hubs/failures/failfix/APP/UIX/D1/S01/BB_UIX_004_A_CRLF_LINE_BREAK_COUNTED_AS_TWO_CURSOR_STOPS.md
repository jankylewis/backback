# BB-UIX-004 — fix

id: BB-UIX-004
family: UIX
status: FIXED
source: PR #41

## The fix that holds

`\r\n` is one line break in the text model: Left and Right step over the
pair, Backspace and Delete remove it whole, and `set_cursor` / `move_to`
never leave a cursor between `\r` and `\n` (it goes before the `\r`). The
word moves that came after (BB-UIX-005) are built on the same rule.

## The wrong fix

Converting the file to `\n` on open (a save would change its line endings),
or special-casing Right alone.

## Proof

Red first: 3 `Document` tests (step over, delete whole, no cursor inside).
Readback in the release build on `class A\r\n{\r\n}\r\n`: the bytes after
Right, `x`, Cmd+S are `class A\r\nx{\r\n}\r\n`; before the fix
`class A\rx\n{`. Byte dumps in `hubs/bugs/bugfix/C1/D1/S03/BB_BUG_010_*.md`.
