# Triage — bugs

One row per bug (`BB-BUG-*`), shelved by cabinet as `bugfacing/`: ids 001-064
in `triage/C1.md`, 065-128 in `triage/C2.md`, and so on. Failure reasons and
tool gaps: `../failures/TRIAGE.md`. Search every shelf with a phrase from your log:

`grep -rn -F '<text>' hubs/bugs/TRIAGE.md hubs/bugs/triage`

| Shelf | Ids | Rows |
|---|---|---|
| [triage/C1.md](triage/C1.md) | BB-BUG-001 to 064 | 48 |
| [triage/C2.md](triage/C2.md) | BB-BUG-065 to 128 | 30 |

A new row goes in its cabinet's shelf, by id; a new cabinet gets its file and
a row here (`hubs/bb_SCHEMA.md`, Triage).
