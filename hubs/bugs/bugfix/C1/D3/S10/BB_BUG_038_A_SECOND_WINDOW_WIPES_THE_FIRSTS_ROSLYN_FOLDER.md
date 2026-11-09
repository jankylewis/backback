# BB-BUG-038 — fix

id: BB-BUG-038
family: PRC
status: FIXED
source: `bb_crates/shell/desktop/src/intel/bb_worker/mod.rs` (`intel_folder`), `bb_crates/bb_core/process/src/temp.rs` (`create` doc)

## The change

Each worker owns `backback-intel-<pid>-<n>` (`OwnedTempDir::create_with`
with a per-process counter); the sweep already reads that form. The
single-name `create` says it is one per process.

## Proof

Unit test `each_worker_owns_its_own_folder`: red on the old code (both
workers got `backback-intel-<pid>`), green after. Gate green with the
server-thread test in it.
