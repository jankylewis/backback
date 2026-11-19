# BB-PRF-008 — The Rust and Python grammars grew the binary by a tenth

id: BB-PRF-008
family: PRF
status: ACCEPTED
date: 2026-11-20
signature: binary size 14.23 MB -> 16.05 MB (+12.8 %) after the language picker
source: `docs/plan/bench/bb_GENERAL/RERUNS/BBEOD37_BINARY_SIZE_AND_DNAAX.md` (section 1)
source: `bb_crates/Cargo.toml` (`tree-sitter-rust = "0.24"`, `tree-sitter-python = "0.25"`, BBEOD26)
observed: BBEOD36 A/B bundles; BBEOD37 linker maps of `aa1d223` and `243c495`
links: fix=no lesson=yes evidence=yes toolgap=none

## What it is

Of the +1,820,792 B the linker placed between `aa1d223` and `243c495`,
the two new tree-sitter grammars hold 1,572,862 B (86 %): their C parse
tables (`__TEXT,__const`) are 1,088,794 B for Rust and 434,380 B for
Python. backback's own crates add 129,332 B (7 %), generic library code
built for them most of the rest. No crate was duplicated, no version
moved, and no test-only feature reaches the release build.

## How it shows

The release binary and the `.app` grow ~1.8 MB; first frame, RSS and
keys do not (BBEOD36 record).

## How to confirm it

Build with `-C link-arg=-Wl,-map,<file>` (the bench record's commands),
sum symbol sizes by crate or archive, compare two commits. `size -m` on
`libtree-sitter-<lang>.a` in the build folder gives each grammar alone.

## Ruling (matemaster, 2026-11-20, BBEOD38 brief): ACCEPTED

"BB-PRF-008: ACCEPTED (+1.8 MB are the Rust/Python grammars = the
feature)." The grammars are the Rust and Python colours themselves; no
size budget is set. No fix file.
