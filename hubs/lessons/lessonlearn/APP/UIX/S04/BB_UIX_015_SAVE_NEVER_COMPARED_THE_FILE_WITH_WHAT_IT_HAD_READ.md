# BB-UIX-015 — lesson

id: BB-UIX-015
family: UIX
status: FIXED
source: night bug loop 2026-11-04

## Who believed what

Save is safe: it keeps the file's mode, owner, links and attributes.

## What changed the belief

A save in a folder other tools also write (git, formatters, agents)
silently undid their change.

## Rule left behind

A write to a file the user did not just read must check it is still the
file that was read; data loss is never silent.
