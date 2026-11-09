# BB-BUG-010 — Right at a line end splits a CRLF line break

id: BB-BUG-010
family: UIX
status: FIXED
date: 2026-11-03
signature: a lone \r in a saved CRLF file
source: CI run 41532817663 led to the edit model; readback in `/Applications/backback.app` at `98f0fcd`
cause: BB-UIX-004
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file with CRLF line breaks (common in C# repositories from Windows);
put the cursor at the end of a line; press Right; type `x`; Cmd+S.

Expected: the cursor goes to the start of the next line; the file keeps
`\r\n`.
Actual: the cursor stops between `\r` and `\n` (line 1, col 9) and the
saved file reads `class A\rx\n{`. Backspace at a line start removed only
the `\n` (a lone `\r` stays); Delete at a line end only the `\r`.

Goal: BBEOD02 (editor).
