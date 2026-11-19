# BB-PRF-009 — A first open of a many-folder tree still waits for the walk

id: BB-PRF-009
family: PRF
status: OBSERVED
date: 2026-11-20
signature: first_frame_ms ~220 on the first open of a folder with thousands of sub-folders (no saved choice)
source: `docs/plan/bench/bb_GENERAL/RERUNS/BBEOD37_BINARY_SIZE_AND_DNAAX.md` (section 2)
source: `bb_core/lang/src/detect.rs` `detect_until` (one-thread `ignore` walk)
source: `shell/desktop/src/app/bb_launch/open/langs.rs` `resolve` (joins the early walk before the first frame)
observed: dna-ax tree, BBEOD37: base 184 ms, fix 350 ms exec to first frame; saved choice 184 vs 184
links: fix=no lesson=yes evidence=yes toolgap=none

## What it is

BB-PRF-007 starts the walk right after the arguments and joins it in
`build_window`. The app reaches its first frame ~56 ms after start; a
walk longer than that still holds the window for the rest. The dna-ax
tree's walk is 229 ms: 13,178 entries in 4,873 folders, ~47 µs a folder
(open, read, ignore files), so folders, not files, set its cost. A
saved choice ends the walk at the first file of each ticked language,
so only the first open of a folder (or one never walked to its end)
pays it.

## How to confirm it

Launch the base and the fix bundle on the tree with an empty state
folder, then with a saved choice; time `bb_lang::detect` on it with a
small release program and compare with the growth of `first_frame_ms`.

## Not changed

Options for a ruling: walk on several threads (`build_parallel`, 4
threads: 68 ms on dna-ax), or a first frame that does not wait for the
walk (ticks shown when it ends).
