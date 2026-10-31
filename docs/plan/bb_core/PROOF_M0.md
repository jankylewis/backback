# 05 · M0 proof: run it, record it

M0 proves the Rust CLI, its conventions, and its process path to Roslyn and the
.NET SDK. Record how long a real solution takes to load.

## Prerequisites

```sh
# Rust toolchain (stable ≥ 1.85) and the gate tools
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
cargo install cargo-deny

# .NET SDK 10 is already on this machine (dotnet --version → 10.0.403)
# Official Roslyn language server (MIT, prerelease)
dotnet tool install --global roslyn-language-server --prerelease

# Debugger (MIT). macOS/Linux builds on the GitHub releases page:
#   https://github.com/Samsung/netcoredbg/releases  → put `netcoredbg` on PATH
```

## Step 1 · Gate

```sh
cd backback/bb_crates
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

Record: number of tests passed per crate, clippy warnings (must be 0), deny
result. This skeleton was written without a Rust toolchain on the authoring
machine; **the first run of this gate is part of M0**, and any compile error it
finds is fixed before anything else.

## Step 2 · Doctor

```sh
cargo run -p bb -- doctor
```

Expected lines: `theme`, `light ok`, `dotnet`, `sdk 10.0.403`, `roslyn-ls <path>`,
`netcoredbg <path or MISSING>`. A MISSING debugger does not block M0.

## Step 3 · Open a real solution and time it

```sh
cargo run --release -p bb -- open ../../MbTx/MbTx.slnx
cargo run --release -p bb -- open ../../dna-nemocatalog-tx/NmcTx/NmcTx.slnx
```

Lines to record (paste verbatim into the table below):
`projects`, `initialize`, `loaded … until workspace/projectInitializationComplete`,
`total`. Run each three times; keep the median. Note machine, OS, SDK version,
Roslyn version (`roslyn-language-server --stdio --version`).

| Solution | .cs files | projects | initialize | loaded | total | run date |
|---|---|---|---|---|---|---|
| MbTx | 2,080 | | | | | |
| NmcTx | 279 | | | | | |
| hello world | 1 | | | | | |

## Step 4 · Local syntax speed

```sh
cargo run --release -p bb -- outline ../../Apex/src/Apex/<a large file>.cs
```

Record `parsed: N bytes in T`. Target: ≤ 5 ms for 50 kB. This is the number that
guarantees highlighting never waits on Roslyn.

## Step 5 · Decide

M0 passes when:
1. The gate is green (fmt, clippy, test, deny) on the same tree.
2. `bb doctor`, `scan`, `open`, `outline`, `audit`, and `bench` pass their fixture
   checks; `bb open` reaches `projectInitializationComplete` on MbTx and records time.
3. The first benchmark baseline and readback round are recorded.

Then M1 starts with the GPUI pin (ADR-2). If `bb open` cannot load MbTx, the
blocker is in Roslyn or the SDK, not in backback, and it is diagnosed from
the Roslyn log directory (`$TMPDIR/backback-roslyn-logs`) before any UI work.

## Honest notes
- Roslyn's load time is Roslyn's; backback can only start it earlier and show
  progress. Publish the number anyway.
- `bb open` replies `null` to every server request (e.g. progress token
  creation). That is enough for loading; M1 implements the real handlers.
- No GUI exists yet. Anyone reading "IDE" at M0 should read "proof of the engine".
