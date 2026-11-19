# BB-UIX-016 — lesson

id: BB-UIX-016
family: UIX
status: FIXED
source: night bug loop 2026-11-04

## Who believed what

What the tree read is what is on disk.

## What changed the belief

Three bugs in one night had one shape: a save (BB-BUG-034), a tab
(BB-BUG-035) and the tree (BB-BUG-037) each trusted their first read
while other tools changed the files.

## Rule left behind

Anything shown from disk is checked against the disk again; a folder
others write to is never read once.
