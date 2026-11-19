# BB-ENV-003 — The fallback PATH missed rustup's folder

id: BB-ENV-003
family: ENV
status: FIXED
date: 2026-11-19
signature: `rust-analyzer not installed` from a launchd start whose login shell gave no `PATH`
source: `bb_crates/bb_core/process/src/shell_env.rs` (`fallback_dirs`); BB-BUG-078
observed: readback `env -i`, launchd `PATH`, `SHELL` naming no shell: Rust missing, Python Ready
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

BB-ENV-001's second layer adds folders when the login shell gives
nothing. It listed the .NET and Homebrew folders only; BB-ENV-001 itself
names `~/.cargo/bin` as a shell-only folder. rust-analyzer discovery
(`PATH`, then `rustup which`) found neither the server nor `rustup`.

## How to confirm

Start the app with `env -i HOME=.. PATH=/usr/bin:/bin:/usr/sbin:/sbin
SHELL=/nonexistent` on a Cargo folder; read `rust_server_status=`.
