# BB-TG-011 — a rebase can push a folder over its five-entry cap

id: BB-TG-011
for: none
status: OBSERVED
date: 2026-11-03
source: CI run 41506158277 (PR #30): `docs/plan/readback/FIXES/S03` had 6 entries

Two branches each added one readback folder to the same shelf. Each branch
alone passed the audit; after #30 was rebased onto master the shelf held
six entries and CI's self-audit failed. Locally only the changed crate's
tests were rerun after the rebase, not the audit.

Workaround: after every rebase, run `cargo run -p bb -- audit ..` (or the
whole `gate.sh`) before the push; start a new shelf when one is full.
