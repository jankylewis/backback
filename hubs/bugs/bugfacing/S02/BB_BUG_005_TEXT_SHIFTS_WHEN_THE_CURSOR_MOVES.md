# BB-BUG-005 — Text shifts one column when the cursor moves

id: BB-BUG-005
family: UIX
status: OBSERVED
date: 2026-11-01
signature: text after the cursor shifts right
source: `bb_crates/shell/desktop/src/editor/view/bb_line.rs:51` and `:68`
observed: headmate screenshot: `CapturedOut│boundRequest(`, the line wider by
  one column at the cursor
cause: BB-UIX-001
links: fix=no lesson=no evidence=no toolgap=none

## Steps

1. Open any `.cs` file. 2. Press Left or Right in a line.

Expected: only the cursor moves; every character stays in its column.
Actual: the text after the cursor moves right by one column, and moves back as
the cursor passes.

## Severity

Critical: editing reads wrong text positions; it also breaks column math
for mouse clicks and popups. Goal: next editor fix, before BBEOD08.
