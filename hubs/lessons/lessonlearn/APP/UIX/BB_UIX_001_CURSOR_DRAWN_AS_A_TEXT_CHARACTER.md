# BB-UIX-001 — lesson

id: BB-UIX-001
family: UIX
status: FIXED
source: `bb_crates/shell/desktop/src/editor/view/bb_line/caret.rs`

## Who believed what

Drawing the caret as a char in the text was "simple", and every other piece
assumed 7.8 px per char.

## What changed the belief

The headmate saw the word split at the caret; the window tests with a tab and
an emoji showed the constant width was wrong for clicks and popups too.

## Rule left behind

Nothing that is not in the file goes into the text runs. Every x on a line
comes from the one shaped line the row paints, never from a constant.
