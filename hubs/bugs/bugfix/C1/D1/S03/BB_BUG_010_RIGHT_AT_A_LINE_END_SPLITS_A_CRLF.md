# BB-BUG-010 — fix

id: BB-BUG-010
family: UIX
status: FIXED
source: `bb_crates/bb_core/bb_text/src/document/movement/mod.rs`, `edit/mod.rs`, `mod.rs`

## The change

`\r\n` is one line break in the text model: Left and Right step over the
pair, Backspace and Delete remove it whole, and `set_cursor` / `move_to`
never leave a cursor between `\r` and `\n` (it goes before the `\r`).

## Proof

Red first: 3 `Document` tests (Right/Left step, Backspace/Delete, no cursor
inside the pair). Readback, release build, scratch file `class A\r\n{\r\n}\r\n`,
launchd environment: Right at the end of line 1 -> line 2, col 1; `x` +
Cmd+S -> bytes `class A\r\nx{\r\n}\r\n`; Backspace at the start of line 2 +
Cmd+S -> `class A{\r\n}\r\n`. Before the fix the same keys wrote
`class A\rx\n{`.
