# BB-BUG-078 — A Dock start without a shell PATH missed rust-analyzer

id: BB-BUG-078
family: ENV
status: FIXED
date: 2026-11-19
signature: `rust_server_status=rust-analyzer not installed: rustup component add rust-analyzer` from an app started by launchd
source: BBEOD29 readback with `env -i`, launchd's `PATH` and a login shell that gives nothing; `bb_crates/bb_core/process/src/shell_env.rs` (`fallback_dirs`)
cause: BB-ENV-003
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Start the app the way launchd does (`PATH=/usr/bin:/bin:/usr/sbin:/sbin`,
no `TERM`) with a login shell that fails or gives no `PATH`; open a folder
with a Cargo workspace and a pytest root.

Expected: `Rust: Ready` (rust-analyzer installed by rustup).
Actual: `rust-analyzer not installed: rustup component add rust-analyzer`;
Python was Ready (pyright is in `/opt/homebrew/bin`, on the fallback list).
With a working login shell both were Ready before the fix too.
