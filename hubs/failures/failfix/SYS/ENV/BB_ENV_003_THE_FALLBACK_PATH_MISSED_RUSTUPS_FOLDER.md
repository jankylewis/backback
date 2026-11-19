# BB-ENV-003 — fix

id: BB-ENV-003
family: ENV
status: FIXED
source: `fallback_dirs` adds `~/.cargo/bin`

## The fix that holds

The fallback holds the folder each install command backback prints puts
its tool in: .NET, Homebrew (`npm install -g pyright`), and rustup's
`~/.cargo/bin`. pytest runs as `python3 -m pytest` (user site), no folder.

## Proof

`the_fallback_path_finds_rust_analyzer_where_rustup_installs_it` red
before, green after; readback before/after: Rust missing → Rust Ready.

## The wrong fix

A personal toolchain path in the repo, or telling the user to start the
app from a terminal.
