# BB-BUG-074 — The editor's test menu opened below the test

id: BB-BUG-074
family: UIX
status: FIXED
date: 2026-11-18
signature: right-click on code after a test method (another method, the class's brace): Run / Debug for that test opens
source: headmate (2026-11-18): the menu must open only inside a spec method
cause: `test_marks::holding` took the nearest test method declared on or above the line, with no end
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a spec file, right-click the class's closing brace after the last test.

Expected: No test menu.
Actual: Run / Debug of the test above opens.
