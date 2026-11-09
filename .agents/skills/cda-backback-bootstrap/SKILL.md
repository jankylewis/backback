---
name: cda-backback-bootstrap
description: "Start any backback task: load the backback skill family, read the laws, the triage tables and the code map before touching code."
metadata:
  source: skillhub
---

# cda-backback-bootstrap

Start any backback task: load the backback skill family, read the laws, the triage tables and the code map before touching code.

- Read [AGENTS.md](../../../AGENTS.md) in full: laws, Git grant, conventions, gate, loops, reporting. It overrides every skill.
- Read [hubs/TRIAGE.md](../../../hubs/TRIAGE.md) and [hubs/bugs/TRIAGE.md](../../../hubs/bugs/TRIAGE.md) before any diagnosis; grep the signature first, a known cause is never re-investigated.
- From `bb_crates/` run `cargo run -q -p bb -- map`, then `map <path>` or `map --find <name>`; open only the lines it names, never whole files.
- Pick the next skill: code change -> cca-backback-develop; a user report or red test -> cca-backback-bugfix; hub entries -> cca-backback-hubs; green gate to installed app -> cca-backback-ship; use the app as the user -> cca-backback-readback; speed -> cca-backback-perf.
- Check the primary checkout is on `master` and clean before starting; work happens in `../backback_wt/<topic>` worktrees only (`backback_wt/netcoredbg` is the netcoredbg checkout, never a worktree to remove).
- Scope: a pure C# IDE, macOS only, lightweight. Build only what the headmate asks; propose extras, never build them unasked.
- Load with `agentfarm skillload <seat> cda-backback-bootstrap`.
- Codex consumes this variant: run long steps (gate, CI wait, install) as separate commands and read their exit codes; never chain a commit after a grep.

## Guardrails

- Never read, print or copy `.env` files, auth stores or secrets; runsettings secrets are counted, never shown.
- Real solutions (MbTx, Apex, dna-nemocatalog-tx, msi, mm, swqc) are read-only: backback runs on them with `--no-write`; every writable run uses a scratch copy.
- Only stop processes this session started, one pid at a time; never `pkill`, never the user's running backback.
- One heavy job at a time (gate, build, install); tests with `--test-threads=2` locally.
- netcoredbg patches stay local; ask before any upstream netcoredbg patch or public release tag.
