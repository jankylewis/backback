# BB-GAT-013 — lesson

id: BB-GAT-013
family: GAT
status: FIXED
source: BBEOD28 READY note "written, never run"; BBEOD29 first real run

## Who believed what

A green variant test on a machine without the tool says the present
branch is right.

## What changed the belief

The first machine with pyright turned it red in one run.

## Rule left behind

A variant test's green counts for the branch it took; prove the other
branch with `BACKBACK_REQUIRE_TOOLS=1` on a machine that has the tool.
