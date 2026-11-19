# BB-ENV-003 — lesson

id: BB-ENV-003
family: ENV
status: FIXED
source: BBEOD29 readback, 2026-11-19

## Who believed what

A Dock start finds every tool, since the login shell's `PATH` is adopted.

## What changed the belief

The Dock readback with a failing shell: the fallback list was written
for .NET (BB-ENV-001) and never grew with Rust and Python.

## Rule left behind

A new external tool adds its install folder to the fallback list and is
read back with a Dock start whose shell gives nothing.
