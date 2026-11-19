# BB-BUG-077 — A failed Rust or Python request said C# server

id: BB-BUG-077
family: LSP
status: FIXED
date: 2026-11-19
signature: `C# server: ` in the status after F12 or hover in a `.rs` or `.py` tab
source: `bb_crates/shell/desktop/src/intel/glue/answers/mod.rs` (`Answer::Failed` arm) on `master` b9f89e1
cause: BB-LSP-010
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

In a window with Python (or Rust) active, press F12 in a `.py` tab while
pyright fails the request (for example a timeout).

Expected: `Python server: <reason>`.
Actual: `C# server: <reason>`, pointing the user at Roslyn.
