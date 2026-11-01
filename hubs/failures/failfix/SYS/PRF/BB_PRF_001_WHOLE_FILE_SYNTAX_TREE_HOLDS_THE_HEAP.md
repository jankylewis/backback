# BB-PRF-001 — fix

id: BB-PRF-001
family: PRF
status: FIXED
source: commit `03a5511`

## The fix that holds

Files over 4 MiB parse only ±1,000 lines around the view (tree-sitter included
ranges, absolute offsets); the window follows scrolling. Smaller files unchanged.

## Proof

Red first: `a_large_file_keeps_a_tree_bounded_by_the_window_not_the_file`.
A/B medians: RSS 1,008,144 -> 165,904 KiB (-83.5%); open 2,706 -> 96 ms;
UI work per key p99 2,466 -> 230 us. Gate 127 passed / 0 failed.

## Known limit

Lines outside the window show plain until the next parse.
