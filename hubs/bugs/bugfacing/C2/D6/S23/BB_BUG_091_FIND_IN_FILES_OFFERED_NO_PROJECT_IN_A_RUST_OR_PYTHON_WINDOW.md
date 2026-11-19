# BB-BUG-091 — Find in Files offered no Project in a Rust or Python window

id: BB-BUG-091
family: UIX
status: FIXED
date: 2026-11-20
signature: Find in Files places `Workspace · ws`, `Folder · alpha/src` and no `Project · …` in a Cargo or Python workspace
source: BBEOD40 sweep; `bb_quick_open/bb_model/places.rs:223` `project_of` and `find.rs:70` kept only `.csproj` at 88177da
cause: BB-UIX-025
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open a Rust-only folder with crates `alpha` and `beta`; open `alpha/src/lib.rs`; press Cmd+Shift+F; read the places.

Expected: `Project · alpha · alpha` and `Project · beta · beta`, as `Project · …` for each `.csproj` in C#.

Actual: Only the workspace and the folder: the Project scope button lists nothing.
