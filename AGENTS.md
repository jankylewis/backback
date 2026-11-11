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
- One local browser runner at a time; 1 build job and 1 test thread on the
  headmate's Mac (`gate.sh` sets both): more overheated it (2026-11-11).
- Scope: pure C# IDE, lightweight, minimal features; build only what the
  headmate asks for; propose extras, never build them unasked.
- Hubs (critical): every bug report, red gate, CI failure, flake or false
  claim gets its entries in `hubs/` (schema `hubs/bb_SCHEMA.md`) in
  the same PR as the fix; every feature gets its `hubs/features/` row in
  the PR that ships it. Read `hubs/failures/TRIAGE.md` and `hubs/bugs/TRIAGE.md` before any diagnosis.
- Never write Claude/Codex auto-memory; handoff state goes to
  `docs/plan/PROGRESS.md` or the external ctx hub.

## Git (Rule 63 grant for this repo only)

Peaker granted self-merge on github.com/jankylewis/backback (2026-10-31).
Mates work on a branch per BBEOD (`bbeod-NN-<topic>`) and commit there as
`jankylewis <jankylewis.se@gmail.com>`; never push to `master`. Only
`backback_mm` opens and merges the PR, after the gate and CI are green.
This grant does not extend to any other repository.

## Conventions (enforced by `bb-audit`; `cargo test` fails on a violation)

- HD taxonomy: at most 5 visible entries per folder (dotfiles and the agent
  files `AGENTS.md`/`CLAUDE.md` excepted). The skill trees `.claude/` and
  `.agents/` keep their tool's layout (no folder rules) but keep the line caps.
- Skills: the backback family lives in `.claude/skills/cca-backback-*` and
  `.agents/skills/cda-backback-*`; start every task with `*-backback-bootstrap`.
- Caps: `.md` ≤ 100 lines; `.rs`/`.cs` ≤ 250; function ≤ 50; Rust `impl`/type
  and C# class ≤ 200; parameters ≤ 5.
- Backbone marker: C# trees use `_` for the one most critical entry per folder;
  Rust and neutral trees use `bb_`. A Cargo crate's `src/` needs no marker.
- Explicit types: every Rust `let` and closure parameter is typed; C# never
  uses `var`. No `unwrap`/`expect`/`panic`/`as` casts/`[i]` outside tests.

## Code map (read it before the source)

Start every task with the map, not with whole files. From `bb_crates/`:
`cargo run -q -p bb -- map` (one line per crate, ~600 tokens), then
`map <path>` (each file's purpose and its public items with line numbers;
`--all` adds private ones), or `map --find <name|Owner::name>`. Then open
only the lines it names. The map is read from the source on every call, so
it is never stale; nothing to regenerate.

## Gate (all must pass, quote real counts)

One command: `docs/release/scripts/gate.sh` (sets PATH, netcoredbg, one
worker). By default it checks what the branch changed: `cargo fmt --check`
and `bb audit` always; clippy (`-D warnings`) and tests for each changed
crate and every crate that uses it; the whole workspace and `cargo deny`
when `Cargo.toml`, `Cargo.lock` or `.cargo/` changed (BB-TG-023).
`gate.sh --plan` shows the scope; `gate.sh --full` checks everything; use it
when unsure. CI runs the full suite on every PR, so a merge is never less.

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
