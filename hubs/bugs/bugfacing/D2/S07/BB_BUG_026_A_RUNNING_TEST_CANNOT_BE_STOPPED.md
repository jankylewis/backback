# BB-BUG-026 — A running test cannot be stopped

id: BB-BUG-026
family: UIX
status: FIXED
date: 2026-11-03
signature: after ▶ there is no stop; the run must end by itself
source: the headmate's report 2026-11-03
cause: BB-UIX-013
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Run a test with its ▶ and try to stop it.

Expected: a stop control where the run started, that ends the run now.
Actual: the headmate found none. A `Stop` link did exist, in the toolbar
above the filter, in the same colour as every other link, shown even with
nothing running.
