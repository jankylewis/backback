# BB-BUG-066 — A debugged test never got its ✓ or ✕

id: BB-BUG-066
family: TST
status: FIXED
date: 2026-11-17
signature: after Debug on a test runs to its end, its TESTS row keeps · (no ✓, no ✕)
source: headmate (MbTx, ChipKeyboardEditModeSpec, 2026-11-17)
cause: the session attached to the paused test host and, at its end, stopped the run (`host.stop()`) without reading its results; plain projects passed no TRX logger at all
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Debug on a test in TESTS, let it run to its end (pass or fail).

Expected: the row shows ✓ (passed) or ✕ in red (failed), as after a run.
Actual: the row keeps ·.
