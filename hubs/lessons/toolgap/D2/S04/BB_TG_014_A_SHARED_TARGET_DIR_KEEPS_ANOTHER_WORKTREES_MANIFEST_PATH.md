# BB-TG-014 — A shared target dir keeps another worktree's manifest path

id: BB-TG-014
for: none
status: OBSERVED
date: 2026-11-03
source: the first gate of branch `bbeod-21-search-dot-folders`, run with `CARGO_TARGET_DIR` set to the main checkout's `.target`

To skip a from-scratch build per worktree, gates ran with `CARGO_TARGET_DIR`
pointing at one shared `.target`. Cargo reuses a test binary whose sources
hash the same, whichever worktree compiled it, and `env!("CARGO_MANIFEST_DIR")`
is baked in at compile time. The e2e tests of `bb` build their fixture paths
from it, so once the worktree that compiled them was removed (after its
merge), 20 tests failed with `cannot read .../backback_wt/<gone
worktree>/.../fixtures/_Harness: No such file or directory`. Nothing in the
branch under test was wrong.

## Workaround

Before a gate in a shared target dir, `touch` the worktree's `*.rs` files:
every workspace crate rebuilds with its own manifest path, dependencies stay
cached. Or give each worktree its own target dir (one full build each).

## Status

OBSERVED. `gate.sh` could refuse a `CARGO_TARGET_DIR` outside the worktree,
or the tests could resolve fixtures at run time; proposed, not built.
