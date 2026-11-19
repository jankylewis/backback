# BB-BUG-079 — An empty rename said Renamed: 0 edits

id: BB-BUG-079
family: LSP
status: FIXED
date: 2026-11-19
signature: `Renamed: 0 edits in 0 files`
source: BBEOD29 window test on the real pyright; `bb_crates/shell/desktop/src/intel/glue/answers/mod.rs` (`Answer::Rename`)
cause: BB-LSP-011
links: fix=yes lesson=yes evidence=yes toolgap=none

## Steps

Open a Python folder and press F2 on a call right away (pyright still
finding its source files), type a name, Enter, Enter.

Expected: a message that nothing can be renamed yet; try again.
Actual: an empty preview opens and Enter says `Renamed: 0 edits in 0
files`, as if the rename ran.
