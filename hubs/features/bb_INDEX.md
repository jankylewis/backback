# Features hub

Every feature backback ships has one row here, in the same PR that ships it
(law: `bb_SCHEMA.md`). Entries: 78. Bugs a user met are in `bugs/`; this hub
lists what the app can do and when it came.

| Shelf | Ids |
|---|---|
| `S01.md` | BB-FEAT-001 to 040 |
| `S02.md` | BB-FEAT-041 to 078 |

## How to add a feature

1. Take the next id after the last row of the last shelf.
2. Add its row to that shelf (40 rows a shelf; a full shelf starts `S<nn+1>.md`
   and a row here).
3. Columns: the feature in the user's words, the PR, the merge date, and the
   bug or tool gap id it closed (`none` if it closed none).
4. A fix that only repairs what worked before is a bug, not a feature.
