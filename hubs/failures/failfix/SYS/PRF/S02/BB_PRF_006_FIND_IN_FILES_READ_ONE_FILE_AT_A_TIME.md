# BB-PRF-006 — fix

id: BB-PRF-006
family: PRF
status: FIXED
source: `bb_crates/bb_core/search/src/walk/mod.rs` (`search_in`, `walk_sorted`), `walk/bb_scan.rs` (`scan`)

## The fix that holds

The first 1,024 files come from the walk as listed; the rest from a
parallel walk, sorted (a path sorts folder by folder, by name: the
walk's own order) and taken after the last file searched. Files are read
and matched in batches of 128 on up to 8 threads; each batch's hits are
added in file order, so hits, order, files count, cap and stop are as
before.

## Wrong fix

The parallel walk for every file: a search capped after 809 files went
34 -> 48 ms, waiting for the whole tree. The first 1,024 files as listed
keep it at 28-40 ms.

## Proof

MbTx A/B, 3 rounds x 7 runs: Unavailable 220-255 -> 145-165 ms; no match
220-225 -> 147-150 ms; capped 36-44 -> 28-40 ms; same hits and files.
`tests/parallel.rs`: 1,500 files over both phases, caps 0..2,000, names
`a/`, `a-b/`, `a.txt`, equal to one file at a time; red when the merge
order or the phase boundary is broken.
