---
name: cca-backback-bugfix
description: "Take a backback bug from a user report or a red test to its proven cause and a family-wide fix, with every hub entry in the same PR."
metadata:
  source: skillhub
---

# cca-backback-bugfix

Take a backback bug from a user report or a red test to its proven cause and a family-wide fix, with every hub entry in the same PR.

- Answer 'is this a bug?' with evidence: find the code path first (`bb map --find`), then reproduce outside the app on a scratch copy (scratchpad), quoting the exact command and output.
- Name the cause at `file:line` and the mechanism, not the symptom. Separate backback's fault from a tool's limit (e.g. an old Apex CLI that ignores flags) and say which is which in plain words: the bug is backback's when backback asked the wrong tool.
- Sweep the family: every caller of the faulty decision (grep the function), and the other front end (`bb` CLI vs app). Fix them all with one shared rule.
- Name the risk while the bug is live (e.g. 'do not press Run: it would run the whole suite') before fixing; never trigger that risk to prove it.
- Fix through cca-backback-develop (failing test first, mutation check), then record it through cca-backback-hubs, then ship through cca-backback-ship.
- Report to the user: cause, what changed, which test proves it, and that it is installed (installed `BBCommit` = master), never just 'merged'.
- Load with `agentfarm skillload <seat> cca-backback-bugfix`.
- Claude Code consumes this variant: use the Read/Edit tools for source, `run_in_background` for the gate and CI waits, and the scratchpad for temp files.

## Guardrails

- Never read, print or copy `.env` files, auth stores or secrets; runsettings secrets are counted, never shown.
- Real solutions (MbTx, Apex, dna-nemocatalog-tx, msi, mm, swqc) are read-only: backback runs on them with `--no-write`; every writable run uses a scratch copy.
- Only stop processes this session started, one pid at a time; never `pkill`, never the user's running backback.
- Never run a step against a live site or a real solution in write mode to confirm a theory.
