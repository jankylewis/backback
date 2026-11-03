# BB-UIX-014 — lesson

id: BB-UIX-014
family: UIX
status: OBSERVED
source: BB-BUG-031 investigation 2026-11-04

## Who believed what

The headmate, and at first the matemaster: backback lists only C# files.

## What changed the belief

`ls -A` against the tree: every entry matched; the folders held only
`__pycache__`.

## Rule left behind

Before fixing "a file is missing", diff the view against the disk
(`ls -A`). A report of a hidden file is first a question about the disk.
