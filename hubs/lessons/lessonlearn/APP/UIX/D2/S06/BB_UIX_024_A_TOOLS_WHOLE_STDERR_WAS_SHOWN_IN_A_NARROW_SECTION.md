# BB-UIX-024 — lesson

id: BB-UIX-024
family: UIX
status: FIXED
source: BBEOD32 readback row 2a (b), BBEOD33 fix

## Who believed what

The fixture's broken `Cargo.toml` made it look like an artefact.

## What changed the belief

BBEOD33 reproduced it first: the section's height with a real cargo error.

## Rule left behind

A narrow panel shows one line with the place and keeps the whole text one hover away; a tool's Display is for logs, not for panels.
