# BB-PRF-004 — lesson

id: BB-PRF-004
family: PRF
status: FIXED
source: night robustness sweep 2026-11-04

## Who believed what

Typing benches on a 1 MB file were fast (290 us a key), so the editor
scales with file size.

## What changed the belief

The 1 MB bench file has 20 000 short lines; one 229 000-char line took
31 s a key. Cost per frame scales with the longest visible line, not
the file.

## Rule left behind

Bench the shapes real files take (one huge line, many short ones, a
huge folder), and bound every per-frame cost by what fits on screen.
