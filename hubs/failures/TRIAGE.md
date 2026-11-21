# Triage — read first

One row per failure reason and tool gap, shelved by layer next to its entries.
Bugs a user met: `../bugs/TRIAGE.md`. Search every shelf with a phrase from your log:

`grep -rn -F --include=TRIAGE.md '<text>' hubs/failures hubs/lessons/toolgap`

| Shelf | Families | Rows |
|---|---|---|
| [failurereason/APP/TRIAGE.md](failurereason/APP/TRIAGE.md) | UIX, LSP, DAP, TST | 40 |
| [failurereason/SYS/TRIAGE.md](failurereason/SYS/TRIAGE.md) | PRC, PRF, ENV | 22 |
| [failurereason/SHIP/TRIAGE.md](failurereason/SHIP/TRIAGE.md) | BLD, CIP, GAT | 29 |
| [../lessons/toolgap/TRIAGE.md](../lessons/toolgap/TRIAGE.md) | TG (tool gaps) | 9 |

A new row goes in its layer's shelf, by family and id. A shelf near 100 lines
splits by family into `failurereason/<LAYER>/<FAM>/TRIAGE.md` and its row here
names the family files (`hubs/bb_SCHEMA.md`, Triage).
