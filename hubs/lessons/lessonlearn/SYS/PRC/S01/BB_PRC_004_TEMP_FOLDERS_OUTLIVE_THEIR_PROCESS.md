# BB-PRC-004 — lesson

id: BB-PRC-004
family: PRC
status: FIXED
source: branch `bb-res-001-temp-folders`

## Who believed what

Each fixture "starts fresh": it removes its folder before making it.

## What changed the belief

The headmate asked for a leak audit; `$TMPDIR` held 39.5 GB of our
folders. The "fresh" remove targeted this pid's name, which never existed.

## Rule left behind

Every resource a process makes (a child, a folder, a file) has one owner
that removes it, and a sweep that removes what a dead owner left. Audit
the disk, not only processes, when asked about leaks.
