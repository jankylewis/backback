# BB-UIX-021 — lesson

id: BB-UIX-021
family: UIX
status: FIXED
source: BBEOD32 readback (row 3) and `args.rs`'s own doc, BBEOD33 fix

## Who believed what

The C#-only rule was intended (`args.rs` documented it), so the readback marked it a suspect, not a bug.

## What changed the belief

BBEOD33 reproduced it first: a launch of a `.rs` file with `ps` showing no server, against a project that holds `Cargo.toml`.

## Rule left behind

A rule documented in the code can still be the defect: when a feature adds a language, sweep every decision that names the old one (`find_solution`) and give each the new language's question.
