# BB-TG-023 — The gate ran the whole workspace for a one-crate change

id: BB-TG-023
family: GAT
status: FIXED
for: none
date: 2026-11-11

## Gap

`gate.sh` built and tested all 53 suites (580+ tests, many starting
dotnet, Roslyn or netcoredbg) for any change, even a one-file fix in one
crate. With one build job (BB-ENV-002) that took many minutes, while CI
runs the same full suite on every PR anyway. Headmate, 2026-11-11: "why
gate or testing after a bug fix or a feature takes so long? didnt we just
need to run affected test cases?"

## Workaround

`gate.sh` checks what the branch changed: fmt and audit always; clippy
and tests for the changed crates and every workspace crate that uses them
(reverse dependencies from `cargo metadata`); the whole workspace when
`Cargo.toml`, `Cargo.lock` or `.cargo/` changed; docs only, fmt and audit.
`--plan` prints the scope, `--full` runs everything. Proven scopes: a
desktop file -> `backback`; a testkit file -> `bb-testkit`, `bb`,
`backback`; a `bb-text` file -> five crates; a `Cargo.lock` edit -> all.
