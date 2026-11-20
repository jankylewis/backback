# BB-GAT-019 — A test probe matched its own command line

id: BB-GAT-019
family: GAT
status: FIXED
date: 2026-11-20
signature: `left: ["<pid> .../deps/backback-<hash> --exact ...starts_no_pyright_process --nocapture"]` at `opt_in.rs:205`
source: PR #129 CI run 44701010716 (macOS, nextest), failed-log lines 234-251; `lang_tests/tools/python/opt_in.rs` `pyright_processes` at fce43aa
observed: CI only (752 passed, 1 failed); the local `cargo test` run was green
links: fix=yes lesson=no evidence=no toolgap=none

## What it is

A test seam fault, no product fault. The process probe listed this test process and its children from `ps` and kept every row whose command held `pyright` anywhere. nextest runs each test in its own process as `<test binary> --exact <full test name> --nocapture`; the test's name ends `starts_no_pyright_process`, so the probe found the test itself and the "no pyright" assertion failed. `cargo test` runs the binary with no test name on its command line, so local runs never saw it.

## How to confirm it

On fce43aa run `cargo nextest run -p backback -j 1 -E 'test(/tools::python::opt_in::/)'` with `BACKBACK_REQUIRE_TOOLS=1`: it fails at `opt_in.rs:205` and `left` holds the test binary's own row.
