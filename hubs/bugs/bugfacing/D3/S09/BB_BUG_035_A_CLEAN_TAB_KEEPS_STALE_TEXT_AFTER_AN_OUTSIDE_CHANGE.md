# BB-BUG-035 — A clean tab keeps stale text after an outside change

id: BB-BUG-035
family: UIX
status: FIXED
date: 2026-11-04
signature: after `git checkout` or a formatter, the open tab still shows the old text
source: night bug loop 2026-11-04 (follow-up of BB-BUG-034)
cause: BB-UIX-015
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file, change it from outside with no edit in backback, look at
the tab, then type and save.

Expected (VS Code): the tab shows the disk's text within a moment; an
edit lands on it.
Actual: the tab kept the text it read; a save then hit BB-BUG-034's
refusal (before that fix: it wrote the stale text back).
