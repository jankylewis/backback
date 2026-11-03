# BB-UIX-004 — lesson

id: BB-UIX-004
family: UIX
status: FIXED
source: PR #41

## Who believed what

A line break is one char, `\n`, so a cursor can stand at any char index.

## What changed the belief

A CRLF file from a Windows repository: the saved bytes after one Right and
one keystroke had a `\r` on its own.

## Rule left behind

The unit of cursor movement is what the user sees as one thing, not the
byte or the char: a line break is one stop however many bytes spell it.
Every movement and edit is tested on a CRLF fixture as well as an LF one.
