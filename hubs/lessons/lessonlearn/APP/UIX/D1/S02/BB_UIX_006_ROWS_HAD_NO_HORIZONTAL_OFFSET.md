# BB-UIX-006 — lesson

id: BB-UIX-006
family: UIX
status: FIXED
source: branch `bb-bug-013-hscroll`

## Who believed what

The editor had been read back on fixtures whose lines fit the pane and on
real files by line number, so "the editor shows the file" was taken as
proven for wide lines too.

## What changed the belief

The headmate opened a real test file: assertion messages and long method
names ran under the pane's edge, and nothing could bring them back.

## Rule left behind

A readback of an editor surface reaches the two far edges of the text, the
end of the widest line and the last line, by key and by pointer. Fixtures
are made wider and taller than the pane on purpose.
