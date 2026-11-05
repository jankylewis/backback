# BB-BUG-053 — Terminal text could not be selected

id: BB-BUG-053
family: UIX
status: FIXED
date: 2026-11-05
signature: a mouse drag over the terminal highlighted nothing and Cmd+C copied nothing
source: headmate request 2026-11-05 (terminal screenshot: "cannot blackhighlight, plz support")
cause: not built: the panel took keys only
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open the terminal, run a command, drag over its output, press Cmd+C.

Expected: the dragged text highlights and Cmd+C copies it, as in Terminal.app and VS Code.
Actual: nothing highlights; nothing is copied.
