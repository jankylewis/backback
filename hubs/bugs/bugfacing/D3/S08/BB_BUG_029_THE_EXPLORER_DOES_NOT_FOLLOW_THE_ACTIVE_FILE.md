# BB-BUG-029 — The Explorer does not follow the active file

id: BB-BUG-029
family: UIX
status: FIXED
date: 2026-11-04
signature: the active file's row is not shown or not selected in the Explorer
source: the headmate's request 2026-11-04 (VS Code reveals the active file)
links: fix=yes lesson=no evidence=yes toolgap=BB-TG-016

## Steps

Open a file deep in a folder (a click on a test, Go to Definition, Cmd+P),
or switch tabs.

Expected (VS Code): the Explorer opens the folders down to the file,
selects its row and scrolls it into view.
Actual: the tree stayed as it was; the file's folders stayed closed.

No failure mechanism: not built.
