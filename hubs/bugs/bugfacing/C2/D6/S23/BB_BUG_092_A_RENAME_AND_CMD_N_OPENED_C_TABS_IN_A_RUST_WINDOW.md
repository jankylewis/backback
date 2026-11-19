# BB-BUG-092 — A rename and Cmd+N opened C# tabs in a Rust window

id: BB-BUG-092
family: UIX
status: FIXED
date: 2026-11-20
signature: a `.rs` tab in C# colours after a rename touched it closed; `New file` in C# colours with no C# ticked
source: BBEOD40 sweep; `intel/glue/answers/bb_rename.rs:140` `Tab::load(path, tab::CSHARP)`; `untitled.rs:33` `Colours(Lang::CSharp)` at 88177da
cause: BB-UIX-025
links: fix=yes lesson=yes evidence=no toolgap=none

## Steps

Open a Rust-only folder; F2 on `total` in `alpha/src/lib.rs`, a rename that edits closed `math.rs`, Enter. Then Cmd+N.

Expected: `math.rs` opens in Rust colours; the new file is plain text (no C# in the window), C# colours only while C# is active, as before.

Actual: `math.rs` tab and `New file` both in C# colours (C# grammar, C# test scan).
