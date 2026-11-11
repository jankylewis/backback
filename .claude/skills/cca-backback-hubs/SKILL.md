---
name: cca-backback-hubs
description: "Record every backback bug, failure, fix, lesson, tool gap and evidence in hubs/ under its schema, in the same PR as the fix."
metadata:
  source: skillhub
---

# cca-backback-hubs

Record every backback bug, failure, fix, lesson, tool gap and evidence in hubs/ under its schema, in the same PR as the fix.

- Read [hubs/bb_SCHEMA.md](../../../hubs/bb_SCHEMA.md): hubs, layers, families, entry head, statuses, laws.
- A bug (`BB-BUG-nnn`, global numbers) is what the user met: one `bugfacing` and one `bugfix` file at `C<n>/D<n>/S<nn>/BB_BUG_<nnn>_<SLUG>.md`. Shelves hold 4 ids, drawers 4 shelves, cabinets 4 drawers; each folder has a `bb_INDEX.md` and at most 4 entries. Make a new folder level only when an entry needs it, and update every link to moved files.
- A failure (`BB-<FAM>-nnn`) is a mechanism: `failures/failurereason` and `failfix` plus `lessons/lessonlearn` at `<LAYER>/<FAM>/`; a self-detected red gate or CI failure gets one too.
- Rows to add with every entry: the hub `bb_INDEX.md` table row (and its count line), the shelf index row, `bugs/TRIAGE.md` or `hubs/failures/TRIAGE.md` (signature + first check), and `failures/evidence/bb_INDEX.md` when `evidence=yes`.
- Every feature, fix-shipped or not, also gets its `hubs/features/` row (next `BB-FEAT-nnn`) in the same PR.
- Give a new id only after checking both TRIAGE files; ids are never reused.
- Run the audit after writing hubs: `.md` <= 100 lines and the 5-entry rule apply to hubs too.
- Load with `agentfarm skillload <seat> cca-backback-hubs`.
- Claude Code consumes this variant: use the Read/Edit tools for source, `run_in_background` for the gate and CI waits, and the scratchpad for temp files.

## Guardrails

- Public repo: no secrets, e-mails, person names or absolute personal paths; name people by role (headmate).
- Every claim cites a source (path + line, commit, PR, command output). No invented case, no guessed cause: write 'not proven'.
- Never make an empty hub file to complete a set; `links:` says yes only when the file or section exists.
