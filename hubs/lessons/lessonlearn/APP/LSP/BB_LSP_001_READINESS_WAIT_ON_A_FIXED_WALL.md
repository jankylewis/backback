# BB-LSP-001 — lesson

id: BB-LSP-001
family: LSP
status: OBSERVED
source: branch `bbeod-03-diagver` commit `28699aa`

## Who believed what

Red-then-green on rerun was put down to machine load.

## What changed the belief

The project count went past the real number on reload, and the probe needs
about 42 s under load against a 30 s wall.

## Rule left behind

A flake in the gate is a finding. Wait on a named condition, never a fixed
time; count by a distinct key.
