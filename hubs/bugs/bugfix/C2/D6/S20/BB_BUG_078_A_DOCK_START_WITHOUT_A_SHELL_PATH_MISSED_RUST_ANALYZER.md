# BB-BUG-078 — fix

id: BB-BUG-078
family: ENV
status: FIXED
source: branch `fix/real-tools`; `bb_core/process/src/shell_env.rs` `fallback_dirs` adds `~/.cargo/bin`

## The change

The fallback list (used when the login shell gives no `PATH`) adds
rustup's `~/.cargo/bin`, where `rustup component add rust-analyzer`
puts the server and `rustup` itself (the discovery's last step).

## Proof

`the_fallback_path_finds_rust_analyzer_where_rustup_installs_it`: red
before (`left: None`), green after. Readback, same launch before/after:
`rust-analyzer not installed` → `Rust: Ready`, rust-analyzer a child of
the app; pyright Ready both times.
