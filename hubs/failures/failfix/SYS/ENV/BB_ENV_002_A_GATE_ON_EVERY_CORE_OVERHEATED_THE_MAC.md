# BB-ENV-002 — fix

id: BB-ENV-002
family: ENV
status: FIXED
source: `bb_crates/.cargo/config.toml`, `docs/release/scripts/gate.sh`, `.github/workflows/bb_ci.yml`

## The fix that holds

Local builds use one job (`[build] jobs = 1`, and `gate.sh` exports
`CARGO_BUILD_JOBS=1` unless set) and one test thread (`RUST_TEST_THREADS=1`,
`--test-threads=1`). CI sets `CARGO_BUILD_JOBS=3` for its own runner.

## Proof

Two 1-worker gates ran green (578 and 581 passed) without a heat report;
this PR's gate uses the new settings.
