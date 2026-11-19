# BB-PRF-008 — lesson

id: BB-PRF-008
family: PRF
status: OBSERVED
source: `docs/plan/bench/bb_GENERAL/RERUNS/BBEOD37_BINARY_SIZE_AND_DNAAX.md`

## Who believed what

The binary grew with "the Rust and Python code" (BBEOD36, not profiled).

## What changed the belief

A linker map split the growth: 86 % is the two grammars' parse tables,
7 % the code of the new crates.

## Rule left behind

A tree-sitter grammar costs its parse tables, about 0.4-1.1 MB each here:
check `size -m` of its archive before adding a language, and attribute a
size change with a linker map (`cargo bloat` cannot read a stripped
binary), never by guessing from the diff.
