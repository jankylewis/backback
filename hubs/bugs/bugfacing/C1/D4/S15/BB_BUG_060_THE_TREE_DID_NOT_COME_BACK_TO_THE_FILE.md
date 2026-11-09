# BB-BUG-060 — The tree did not come back to the file

id: BB-BUG-060
family: UIX
status: FIXED
date: 2026-11-07
signature: back in a file, its Explorer row stays out of sight (folders open, row below the view)
source: headmate report 2026-11-07 (screenshot: MbTx, active file deep under Sentences/_Ui/Documents)
cause: the Explorer followed a file once per newly active tab (BB-BUG-029); after the user scrolled the tree, clicking back into the same file never showed it again
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a file deep in a tall tree. Scroll the Explorer away (or browse it),
then click back into the file, or on its tab.

Expected: the tree shows the file's row, as VS Code's Explorer does.
Actual: the row stays out of sight until another file is opened.
