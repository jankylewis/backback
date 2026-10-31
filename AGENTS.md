# AGENTS.md — laws for every agent working on backback

backback is a very lightweight, dark-by-default, open-source IDE for C# only,
on macOS only. Rust shell (`bb_crates/`), Roslyn LSP, netcoredbg, `dotnet` CLI.
Fleet: `backback` — matemaster `backback_mm`, mates `backback_1` (cca),
`backback_2` (cda). Plan: `docs/plan/`; EODs BBEOD01..BBEOD07 live in TaskHub.

## Standing laws (from the AgentFarm charter)

- Use the `agentfarm` CLI for fleet actions; a refusal (exit 2) is the system
  working: report it, never work around it.
- `.env` files and auth stores are blacklisted: never read, print or copy them.
  Never print a secret value; runsettings secrets are counted, never shown.
- Never burn a live Okta/MFA login. Never touch another fleet's seat or tree.
- Off limits (read-only at most): `MbTx/`, `Apex/`, `dna-nemocatalog-tx/`, `msi/`,
  `mm/`, `swqc/`. Readback on those solutions is READ-ONLY.
- Claim before writing a shared tree: `agentfarm claim <label> <path>`.
- One local browser runner at a time; at most 2 test workers per seat.

## Git (Rule 63 grant for this repo only)

Peaker granted self-merge on github.com/jankylewis/backback (2026-10-31).
Mates work on a branch per BBEOD (`bbeod-NN-<topic>`) and commit there as
`jankylewis <jankylewis.se@gmail.com>`; never push to `master`. Only
`backback_mm` opens and merges the PR, after the full gate is green.
This grant does not extend to any other repository.

## Conventions (enforced by `bb-audit`; `cargo test` fails on a violation)

- HD taxonomy: at most 5 visible entries per folder (dotfiles and the agent
  files `AGENTS.md`/`CLAUDE.md` excepted).
- Caps: `.md` ≤ 100 lines; `.rs`/`.cs` ≤ 250; function ≤ 50; Rust `impl`/type
  and C# class ≤ 200; parameters ≤ 5.
- Backbone marker: C# trees use `_` for the one most critical entry per folder;
  Rust and neutral trees use `bb_`. A Cargo crate's `src/` needs no marker.
- Explicit types: every Rust `let` and closure parameter is typed; C# never
  uses `var`. No `unwrap`/`expect`/`panic`/`as` casts/`[i]` outside tests.

## Gate (run from `bb_crates/`, all must pass, quote real counts)

`cargo fmt --check && cargo clippy --all-targets -- -D warnings &&
cargo test --workspace && cargo deny check`

## Loops (every BBEOD)

- Readback: use backback as a real user on fixtures and real solutions
  (read-only) → log issue → failing test first → fix → gate → repeat the
  same steps to confirm → lesson in `docs/plan/readback/`.
- Performance: fixed benchmark set, quiet machine, median of 5 after warm-up
  → compare with baseline (>5% = regression) → profile → one change → re-bench
  → keep only if faster with all tests green → `docs/plan/bench/`.

## Reporting

Post READY with: branch, commits, gate output counts, readback and bench
deltas. Write state to `docs/plan/PROGRESS.md` before any long run.
Write ctx summaries to the fleet ctx hub outside this repo (path given in your brief or env BACKBACK_CTX_HUB), never inside the repo.
