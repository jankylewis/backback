# BB-ENV-002 — A gate on every core overheated the Mac

id: BB-ENV-002
family: ENV
status: FIXED
date: 2026-11-11
signature: headmate: "it overheated my machine"
source: headmate report 2026-11-11, during a full gate
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`gate.sh` ran `cargo` with its default build jobs (one per core) and the
tests with `--test-threads=2`, and `.cargo/config.toml` set
`RUST_TEST_THREADS=2`. Back-to-back gates on the headmate's own Mac heated
it until the headmate asked to stop all work.
