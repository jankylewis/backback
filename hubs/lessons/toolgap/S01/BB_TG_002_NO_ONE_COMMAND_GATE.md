# BB-TG-002 — No one-command gate

id: BB-TG-002
for: none
status: OBSERVED
date: 2026-11-01
source: `AGENTS.md` gate line (five commands chained by hand)

The gate is fmt, clippy, test, deny and `bb audit`, typed by hand each time.
A background run on 2026-11-01 exited 127 (`command not found: cargo`) because
that shell had no `$HOME/.cargo/bin` on PATH. Nothing was tested.

Workaround: export PATH first; print each step's exit code. Wanted: one `bb
gate` command that runs every step with `--no-fail-fast` and one summary line.
