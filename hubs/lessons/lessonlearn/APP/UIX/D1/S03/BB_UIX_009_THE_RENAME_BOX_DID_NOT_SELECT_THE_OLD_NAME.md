# BB-UIX-009 — lesson

id: BB-UIX-009
family: UIX
status: FIXED
source: branch `bb-bug-018-rename-select`

## Who believed what

The rename flow was proven: the window rig renamed `x` to `Total` and the
closed file was saved.

## What changed the belief

The rig pressed Backspace before typing because its symbol was one char;
a real symbol typed over without Backspace came out doubled.

## Rule left behind

A rig drives the surface the way a user does, no helper key a user would
not press; when a helper key is needed to make a test pass, that key is the
bug.
