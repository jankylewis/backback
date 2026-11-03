# BB-BUG-024 — Test groups do not collapse

id: BB-BUG-024
family: UIX
status: FIXED
date: 2026-11-03
signature: a namespace or class row in the Tests panel shows ▾ but a click does nothing
source: the headmate's report 2026-11-03 (RgvMbDesktop.Tests.Auth.Ofsi, OfsiAccessScopeSpec)
cause: BB-UIX-012
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open the Tests side, open a project, click a namespace row or a class row.

Expected: the row folds its children (▸) and opens them again (▾).
Actual: nothing; only the project row folds.

Goal: BBEOD07 (tests panel).
