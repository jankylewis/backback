# BB-BLD-003 — Merged is not installed: name the running build

id: BB-BLD-003
status: FIXED
source: BB-BUG-043

## Lesson

A fix the headmate can use is a fix in the running app. Before answering
"fixed?", read the installed `BBCommit` and check it holds the fix
(`git merge-base --is-ancestor`); say plainly when it does not, and which
build the app shows in its status bar.

## Rule

Every report on a UI bug starts with the build the reporter runs.
