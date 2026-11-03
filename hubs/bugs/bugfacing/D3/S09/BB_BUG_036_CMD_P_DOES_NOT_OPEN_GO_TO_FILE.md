# BB-BUG-036 — Cmd+P does not open Go to File

id: BB-BUG-036
family: UIX
status: FIXED
date: 2026-11-04
signature: Cmd+P does nothing; Go to File opens on Cmd+Shift+O only
source: night bug loop 2026-11-04 (a Go to File probe sent Cmd+P and nothing opened); the matemaster had named Cmd+P to the headmate as a way to open a file before checking the binding
cause: BB-UIX-010
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Press Cmd+P in the editor.

Expected (VS Code): Go to File.
Actual: nothing. Go to File was bound to Cmd+Shift+O, which is VS Code's
Go to Symbol in the file; Go to Symbol is Cmd+F12 here.

## Ruling needed

Whether Cmd+Shift+O moves to Go to Symbol (VS Code) or stays Go to File:
the headmate's call, like Cmd+[ / Cmd+].
