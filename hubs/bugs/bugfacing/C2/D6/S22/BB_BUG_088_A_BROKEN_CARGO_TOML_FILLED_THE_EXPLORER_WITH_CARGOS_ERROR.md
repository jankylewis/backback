# BB-BUG-088 — A broken Cargo.toml filled the Explorer with cargo's error

id: BB-BUG-088
family: UIX
status: FIXED
date: 2026-11-19
signature: `cargo metadata: cargo metadata exited with Some(101): error: key with no value...` over 9 lines
source: BBEOD32 readback row 2a b; `bb_rows/cargo.rs` `section` at b13fcbb
cause: BB-UIX-024
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open a folder whose `Cargo.toml` is `x` (the language fixtures).

Expected: One line: `cargo metadata: Cargo.toml:1: key with no value, expected `=``; the full text on hover.

Actual: The whole stderr tail, wrapped over the tree (176 px in the window test).
