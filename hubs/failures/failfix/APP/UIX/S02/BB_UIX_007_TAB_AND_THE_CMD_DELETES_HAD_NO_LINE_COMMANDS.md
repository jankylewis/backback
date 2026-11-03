# BB-UIX-007 — fix

id: BB-UIX-007
family: UIX
status: FIXED
source: branch `bb-bug-014-015-line-keys`

## The fix that holds

Line-wise commands in the text model (`edit/lines.rs`): delete to the line
start or end, indent and outdent the lines a selection touches as one undo
step (`replace_many`), the selection put back by what moved before each
end. The key map gives Tab, Shift+Tab, Cmd+Backspace and Cmd+Delete their
own arms before the plain-insert default.

## The wrong fix

Special-casing Tab inside the insert path, or indenting with a tab char in
a four-space C# file.

## Proof

Red first: the `lines_tests` unit tests and the two window tests in
`editor/input/tests.rs`; readback in
`docs/plan/readback/V2/SWEEP01/bb_READBACK.md`.
