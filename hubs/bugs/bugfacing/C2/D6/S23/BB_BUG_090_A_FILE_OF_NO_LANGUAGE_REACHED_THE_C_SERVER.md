# BB-BUG-090 — A file of no language reached the C# server

id: BB-BUG-090
family: UIX
status: FIXED
date: 2026-11-20
signature: `C# server: Ready` (or `No solution`) after F12 in `Cargo.toml` or a README; its didOpen in Roslyn
source: BBEOD40 sweep; `intel/glue/bb_route.rs:25` `server_for`: `Some(Lang::CSharp) | None => Server::Roslyn` at 88177da
cause: BB-UIX-025
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open a Rust-only folder; open `alpha/Cargo.toml`; type `x`; press F12. In a C# + Rust + Python window do the same.

Expected: No server gets the file; the status bar says `No language server for this file`.

Actual: The status bar says `C# server: …` in a window with no C#; with C# active, Roslyn gets the open, the edit and the ask for a `.toml`.
