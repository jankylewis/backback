# BB-BUG-065 — The gutter's ▶ never ran its test

id: BB-BUG-065
family: TST
status: FIXED
date: 2026-11-16
signature: the ▶ next to a test method in the editor opens the TESTS panel instead of running the test
source: self-detected while building the editor's right-click Run/Debug (BB-FEAT-084)
cause: `test_marks::target` compared `Fqn::method()` (the full name, `Sample.Auth.OutcomeSpec.Fails`) with the name in the source (`Fails`); they never matched, so every ▶ fell back to opening the panel
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

List a test project in TESTS, open one of its spec files, click the ▶ in
the gutter of a test method.

Expected: that test runs.
Actual: the TESTS panel opens; nothing runs.
