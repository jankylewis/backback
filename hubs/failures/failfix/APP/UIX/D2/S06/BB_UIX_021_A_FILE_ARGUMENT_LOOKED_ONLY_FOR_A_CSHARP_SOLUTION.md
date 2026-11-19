# BB-UIX-021 — fix

id: BB-UIX-021
family: UIX
status: FIXED
source: branch `fix/readback-suspects`

## The fix that holds

One rule, `bb_root::project_root`, for every front end that opens a file (`start_for`: argument, start page, Open...): the file's language decides the root it looks for; Rust asks cargo's `workspace_root` through the `bb_cargo::members` seam (5 s deadline) and takes it only inside the walk; mixed workspaces widen to the outermost other-language root; the walk ends at a git root and before home.

## The wrong fix

Looking for any `Cargo.toml` up to `/`: a stray manifest in the home folder or above a git root would become the workspace (now refused by `the_walk_stops_at_a_git_root_and_never_reaches_home`).
