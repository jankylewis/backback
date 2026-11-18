# BB-GAT-012 — One local build job hid a duplicate CI error

id: BB-GAT-012
family: GAT
status: OBSERVED
date: 2026-11-19
signature: `bb_workspace.rs:78` `left: 2` `right: 1`; `cmd_shift_b_in_a_rust_window_lists_the_compile_error_and_a_click_opens_its_line` (`panels.rs:75`)
source: `bb_crates/edge/cargo/src/project/messages.rs` (`command`: `--all-targets`; `messages`)
source: PR #116 CI: 2 tests red, both on the Broken fixture's one error listed twice
observed: `cargo check --workspace --all-targets --message-format=json` on the fixture: `-j1` 1 error, `-j2` 2 errors with identical JSON (package, target `lib` `beta`, span `beta/src/lib.rs:4:5`)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`--all-targets` compiles `beta/src/lib.rs` twice: as the lib and as its
unit-test build. Both fail with the same error, and cargo's JSON for the
two is identical. With one job (local `.cargo/config.toml` `jobs = 1`)
cargo starts nothing after the first failure, so only one copy prints;
with parallel jobs (CI) both compile and both print. The Build panel
listed every copy.

## How to confirm it

Run the command above on the Broken fixture with `-j1` and `-j2` and
count `compiler-message` errors; or feed `messages` the same line twice.
