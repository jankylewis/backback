# BB-BUG-084 — A Rust or Python file argument opened its own folder

id: BB-BUG-084
family: UIX
status: FIXED
date: 2026-11-19
signature: window title `src`, Explorer shows only `main.rs`, no rust-analyzer or pyright
source: BBEOD32 readback row 3; `app/args.rs` `split_argument` at b13fcbb
cause: BB-UIX-021
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open `py_rust/src/main.rs` by argument (or the Dock with a file), where `py_rust/` holds `Cargo.toml` and `pyproject.toml`.

Expected: The Explorer roots at `py_rust/`; rust-analyzer and pyright start.

Actual: The workspace is `src/`; no language server starts (`args.rs` looked only for a C# solution above the file).
