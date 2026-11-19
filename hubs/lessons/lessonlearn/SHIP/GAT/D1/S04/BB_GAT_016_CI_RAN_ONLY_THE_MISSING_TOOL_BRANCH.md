# BB-GAT-016 — lesson

id: BB-GAT-016
family: GAT
status: OBSERVED
source: BBEOD34, master CI run 44555640505

## Who believed what

A green CI run proved both branches of the variant tests.

## What changed the belief

The tests' durations: 0.02 s for a pyright start is the hint branch.
CI had none of the tools, so it never ran the real one.

## Rule left behind

A test with a tool branch is proven where the tool is installed and
`BACKBACK_REQUIRE_TOOLS=1` is set; CI is that place on every PR.
