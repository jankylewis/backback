# BB-GAT-017 — A test-only module in a source file reached the gate unlinted

id: BB-GAT-017
family: GAT
status: FIXED
date: 2026-11-20
signature: `error: items after a test module` ... `pub(crate) mod walks {`; `called \`Iterator::last\` on a \`DoubleEndedIterator\``; `every_public_head_in_this_repo_is_on_the_map` ... `missing: pub(crate) fn`
source: BBEOD36 gate run 1 (`--full`, branch `perf/lang-no-regression`, self-detected): clippy red on `langs.rs:120` and `tests/early_walk.rs:16-20`; `-p bb-kmap --test views` 1 failed
source: `bb_crates/shell/kmap/tests/views.rs:118-145` (which heads the map check skips)
source: commit 708d624 (PR #124): the shape that passed, `langs.rs:178-201`, `early_walk.rs:20`
observed: one gate run caught all three; the targeted desktop test run before it was green 3/3
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

BB-PRF-007 needed a test-only recorder (which thread walked which folder)
inside a production file. It went in as `#[cfg(test)] pub(crate) mod walks`
in the middle of `langs.rs`, holding `pub(crate) fn of`. Three checks
the targeted run does not do then failed:

1. clippy `items_after_test_module`: production items (`status_line`,
   `change`, `NO_DEBUG`, `may_debug`) followed a `#[cfg(test)]` module.
2. clippy `double_ended_iterator_last` in the new test helper
   (`.windows().into_iter().filter_map(..).last()`): the test target is
   linted only by clippy `--all-targets`, which the gate runs.
3. `every_public_head_in_this_repo_is_on_the_map` (crate `bb-kmap`, not a
   user of `backback`): it skips a file only for `#[cfg(test)]\nmod tests {`
   and a head only when `#[cfg(test)]` is the line right above it. A
   `pub(crate) fn` inside a test module with another name is counted, but
   the map did not list `of` (line 134), so the check failed on it.

## How to confirm it

Grep the gate log for the signature lines. The two checks the targeted
run skips are `cargo clippy -p backback --all-targets -- -D warnings` and
`cargo test -p bb-kmap --test views`; both were red in gate run 1 and
green in runs 2 and 3. A fresh repro on a scratch module was not run.
