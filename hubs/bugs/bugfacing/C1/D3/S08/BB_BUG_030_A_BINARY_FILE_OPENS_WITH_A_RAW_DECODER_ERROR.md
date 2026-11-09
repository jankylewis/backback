# BB-BUG-030 — A binary file opens with a raw decoder error

id: BB-BUG-030
family: UIX
status: FIXED
date: 2026-11-04
signature: document: file is not UTF-8: invalid utf-8 sequence of 1 bytes from index 0
source: readback 2026-11-04 on the installed app at `8392878` (a `.pyc` opened)
links: fix=yes lesson=no evidence=yes toolgap=BB-TG-016

## Steps

Open a binary file (`__pycache__/*.pyc`) from the Explorer.

Expected: a plain sentence that the file is not text.
Actual: the decoder's own detail in the status bar.

No failure mechanism: the error text was the decoder's.
