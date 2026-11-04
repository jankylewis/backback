# BB-BLD-003 — A merged fix was reported before it was installed

id: BB-BLD-003
family: BLD
status: FIXED
date: 2026-11-04
signature: the headmate still sees a bug whose fix is merged; Info.plist `BBCommit` is older than the fix
source: BB-BUG-043
observed: BB-BUG-043
links: fix=yes lesson=yes evidence=yes toolgap=BB-TG-019

## What it is

The fix for BB-BUG-042 merged at 12:12 while the app was open, so it was
not installed. The headmate reopened the app at 13:00, still `be13e35`,
and met the bug again. The app showed no build, so nothing told them
the build was stale; my "merged, install when you quit" was easy to miss.
A repeat of BB-BLD-001's lesson (name the build before reading a report).

## How to confirm it

`/usr/libexec/PlistBuddy -c 'Print :BBCommit' /Applications/backback.app/Contents/Info.plist`,
then `git merge-base --is-ancestor <fix> <that commit>`.
