# BB-BUG-096 — Esc did not close Find results after the terminal had the keys

id: BB-BUG-096
family: UIX
status: OBSERVED
date: 2026-11-26
signature: Esc with the results panel open and the terminal focused: the panel stays
source: headmate report 2026-11-26 (BBEOD45 brief: "make Esc close it")
observed: with the terminal open below, Esc did not close the Find results panel
cause: the panel took no focus; Esc reaches `bb_keys::key` only through the editor's key handler
links: fix=yes lesson=no evidence=yes toolgap=none

## Steps

Open the terminal, run Find in Files, click the results panel, press Esc.

Expected: the panel closes (BB-BUG-047, as its `✕` does).

Actual: the keys stayed with the terminal; the panel stayed open.
