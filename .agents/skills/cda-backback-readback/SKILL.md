---
name: cda-backback-readback
description: "Use the installed backback as its end user on fixtures and real solutions (read-only), find what is wrong, and feed it to bugfix."
metadata:
  source: skillhub
---

# cda-backback-readback

Use the installed backback as its end user on fixtures and real solutions (read-only), find what is wrong, and feed it to bugfix.

- Read the readback records under [docs/plan/readback/](../../../docs/plan/readback/) for the last round and its open items.
- Launch as the Dock does, not with the shell's environment: `env -i` with the launchd PATH, or `open -a` (zsh `BG_NICE` nices `&` launches). Real solutions open with `--no-write`.
- Drive keys with `BB_KEYS="..."` (and `BB_KEYS_QUIT=1`); mouse behaviour is proven by window tests, because posted clicks from the agent shell are dropped and a locked screen paints no frames.
- Keep evidence: commands, state lines, screenshots of the front window only; compare with the expected behaviour stated by the headmate or VS Code/Zed.
- Each finding goes to cca-backback-bugfix (failing test first); after the fix, repeat the same steps on the installed build to confirm, and record the round in `docs/plan/readback/`.
- Load with `agentfarm skillload <seat> cda-backback-readback`.
- Codex consumes this variant: run long steps (gate, CI wait, install) as separate commands and read their exit codes; never chain a commit after a grep.

## Guardrails

- Real solutions (MbTx, Apex, dna-nemocatalog-tx, msi, mm, swqc) are read-only: backback runs on them with `--no-write`; every writable run uses a scratch copy.
- Only stop processes this session started, one pid at a time; never `pkill`, never the user's running backback.
- A readback launch takes the keyboard focus: never run one while the user is typing in another app without saying so.
- Never claim a feature works from tests alone when the user asked about the app; use the installed build.
