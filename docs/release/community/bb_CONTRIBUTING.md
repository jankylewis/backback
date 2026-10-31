# Contributing

backback is a macOS-only, C#-only IDE project. The current M0 build is a CLI.

## Set up

Install Rust 1.85 or newer, the .NET SDK, and `cargo-deny`. Work on a branch.
Read [AGENTS.md](../../../AGENTS.md) and [conventions](../../conventions/CONVENTIONS.md).

## Check a change

Run from `bb_crates/`:

```sh
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo deny check
cargo run -p bb -- audit ..
```

For a defect, reproduce it, add a test that fails on the old code, then fix it.
For a user-facing change, run the same action again on the served app.
Keep a readback note in `docs/plan/readback/`.

For performance work, use the fixed benchmark set on a quiet machine.
Record five runs after warm-up and compare their median with the baseline.
Profile a regression above 5%, make one change, then rerun the benchmark and gate.

## Pull requests

Describe the change, the commands and results, and any readback or bench delta.
Never include secrets, local auth stores, or `.env` files.
