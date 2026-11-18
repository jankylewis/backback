# BB-GAT-011 — The lexer dropped an escaped line break

id: BB-GAT-011
family: GAT
status: FIXED
date: 2026-11-19
signature: `every_public_head_in_this_repo_is_on_the_map` ... `missing: pub fn` in a file with a `\`-continued string
source: `bb_crates/shell/audit/src/lexer.rs` (`skip_string`, `skip_char`: `'\\' => i + 2`)
source: BBEOD27 full gate: suites=63 passed=662 failed=1 (`edge/cargo/src/fixtures.rs:47`)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`strip` blanks literals but must keep every line break. An escape skipped
two chars; when the escaped char was the newline of a Rust string line
continuation (`"a \` then a new line), that break was lost, so every
later line of the stripped text sat one line too high. `bb map` then
put items on wrong lines (and `bb audit` would report wrong lines).

## How to confirm it

A file with N continued string lines: the map's lines after them are
off by N; `strip(src).matches('\n').count()` is below the source's.
