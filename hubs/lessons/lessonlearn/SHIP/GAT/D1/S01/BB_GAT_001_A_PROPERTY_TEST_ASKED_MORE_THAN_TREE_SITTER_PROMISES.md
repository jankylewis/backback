# BB-GAT-001 — lesson

id: BB-GAT-001
family: GAT
status: FIXED
source: branch `bb-syn-incremental`

## Who believed what

Incremental parsing must give the same tree as a fresh parse, always.

## What changed the belief

A shrunk proptest case: equal edit points, different trees only in broken
code, equal again one edit later.

## Rule left behind

A property test asserts the library's documented contract, not a stronger
one; check the inputs we pass (here: edit points) before blaming the library.
