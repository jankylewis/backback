# BB-UIX-025 — A language-neutral path fell back to C#

id: BB-UIX-025
family: UIX
status: FIXED
date: 2026-11-20
signature: `C# server: …` in a C#-less window; no `Project · …` for a crate; a `.rs` or new tab in C# colours
source: `bb_route.rs:25`, `places.rs:223`, `find.rs:70`, `bb_rename.rs:140`, `untitled.rs:33` at 88177da
observed: BB-BUG-090, BB-BUG-091, BB-BUG-092
links: fix=yes lesson=yes evidence=no toolgap=none

## What it is

Code from the C#-only app answered "which language?" with C# when the input had none or another: an `Option<Lang>` matched `None` with C#, a project was a `.csproj`, a loaded tab was `tab::CSHARP`. The sixth shipped member of the family after BB-BUG-075, 077, 084, 085, 086 and BB-LSP-013.

## How to confirm it

In a Rust-only window: F12 in `Cargo.toml`; Cmd+Shift+F in a `.rs` file; a rename that edits a closed file; Cmd+N. Read the status, the places and the tab modes.
