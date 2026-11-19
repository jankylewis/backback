# BB-LSP-012 — A Cargo.lock appeared in the workspace on open

id: BB-LSP-012
family: LSP
status: ACCEPTED
date: 2026-11-19
signature: `Cargo.lock` in a workspace that had none, after the window opened it with rust-analyzer
source: BBEOD29 window test `rust_analyzer_by_mouse_and_keys_f12_click_hover_rename_problems_or_its_install_hint`; `bb_crates/edge/toolchains/bb_cargo/tests/bb_workspace.rs` (`metadata_lists_both_members_without_writing_the_project` asserts no lock for backback's own call)
observed: fixture with no lock: after the window test, `Cargo.lock` at its root, no `target/`. `rust-analyzer analysis-stats` (no check) wrote none; `cargo metadata` without `--no-deps` writes one
links: fix=no lesson=no evidence=yes toolgap=none

## What it is

Not proven which step writes it; the likely one is rust-analyzer's
`cargo check` (flycheck) that gives Problems, which resolves and writes
the lock as `cargo check` in a terminal does. Stable cargo has no flag to
keep the lock elsewhere; `--locked` fails without one.

## Ruling (headmate, 2026-11-19, BBEOD30): ACCEPTED

A `Cargo.lock` written by rust-analyzer in a lock-less Cargo workspace
is accepted: it is standard for every rust-analyzer client, and turning
it off (no-deps) would lose dependency navigation. No fix. The live Rust
window test keeps `Cargo.lock` as its single allowed new file.

## How to confirm

Open a lock-less workspace with rust-analyzer; list its root.
