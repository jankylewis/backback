# BB-GAT-002 — A fixture folder shared by parallel test processes

id: BB-GAT-002
family: GAT
status: FIXED
date: 2026-11-06
signature: fixture build failed ... GotoFixture.runtimeconfig.json because it is being used by another process / project.assets.json not found
source: PR #92, CI run 42191585967 (first run with parallel nextest workers)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

`bb-dap`'s tests build a C# fixture with `dotnet build` in one fixed
folder per tag (`.target/tmp/backback-lifecycle`), once per process (a
`OnceCell`). `cargo test` runs a file's tests in one process, so it held.
`cargo nextest` runs each test in its own process: 4 workers deleted and
rebuilt the same folder at once. 7 of 533 tests failed.

## How it shows

`fixture.rs:36` "fixture build failed" with MSB3191 / NETSDK1004 / a
file in use, or a 120 s build timeout (`Elapsed`). Local repro:
`cargo nextest run -p bb-dap --test lifecycle -j 4`: 5 of 11 failed.
