# BB-UIX-024 — A tool's whole stderr was shown in a narrow section

id: BB-UIX-024
family: UIX
status: FIXED
date: 2026-11-19
signature: `cargo metadata: cargo metadata exited with Some(101): error:` wrapped over the Explorer
source: `bb_rows/cargo.rs` `section` at b13fcbb
observed: BB-BUG-088
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

`CargoMembers::Failed(error.to_string())` put the error's Display (tool, exit code, the last 5 stderr lines) into a 300 px section that wraps; cargo's caret block became 8-9 lines above the tree. The tail of 5 lines can also lose cargo's `error:` line for a long error.

## How to confirm it

Open a folder whose `Cargo.toml` is `x`; read the Cargo section's height.
