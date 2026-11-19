# BB-GAT-018 — lesson

id: BB-GAT-018
family: GAT
status: FIXED
source: PR #127 CI red, BBEOD39 follow-up

## Who believed what

A 3 s hold looked generous next to a local open of ~0.3 s, and closing the window looked like stopping its servers.

## What changed the belief

The CI log's own `editor_open_ms=3093`, and a mutation that left the link in the log: 1 of 4 server threads ended.

## Rule left behind

A test seam never releases on wall clock below what CI load can take; a test that starts a real process waits for it to end before it returns.
