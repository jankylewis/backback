# BB-UIX-014 — A folder of only caches reads as a tree that hides files

id: BB-UIX-014
family: UIX
status: OBSERVED
date: 2026-11-04
signature: a source folder in the Explorer shows `__pycache__` and subfolders, no source files
source: `bb_crates/bb_core/explorer/src/tree/children.rs` (`read_folder` lists every entry but `.DS_Store` and `.git`)
observed: BB-BUG-031
links: fix=no lesson=yes evidence=yes toolgap=none

## What it is

A checkout whose `.py` files were deleted or moved keeps its
`__pycache__` folders (git ignores them, so nothing removes them). The
tree then shows packages with folders only, which reads as a filter.

## How to confirm it

`ls -A` the folder and compare it with the tree row for row before any
change to the listing.
