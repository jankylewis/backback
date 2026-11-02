# Go to File open cost, 2026-11-03 (BB-PRF-002)

By backback_mm, release build, `--no-write`. Input through `BB_KEYS`; the
app prints UI work per key (dispatch + next frame). Host not quiet (load
6-10).

| Tree | Keys | Before: p99 / max | After: p99 / max |
|---|---|---|---|
| Apex (about 1,200 listed files) | 6 x Cmd+Shift+O, Esc | 21 / 27 ms | not rerun |
| Synthetic 50,000 `.cs` files (the walk cap) | 6 x Cmd+Shift+O, Esc | 196 / 204 ms | see next row |
| Synthetic 50,000, typing 9 keys in the box | before: 1 open + 9 keys | p99 5.5 ms (open: 196 ms) | |
| Synthetic 50,000, one run of 22 keys | 7 opens, 6 Esc, 9 typed | | 5.2 / 5.2 ms |

The synthetic tree: 500 folders x 100 one-line `.cs` files plus a `.slnx`,
made in a scratch folder (not in the repo). Correctness after the change:
`p421/sub/c017` + Enter opened `P421/Sub/C017.cs`.
