# BBEOD06 readback: Find in Files (Cmd+Shift+F)

Date 2026-11-03, by backback_mm. Branch `bbeod-06-find`, release build.

## Setup

- App: release `backback`, `--no-write`, opened on a file of the Apex
  solution (read-only); root = the `Apex.slnx` folder, about 9,800 files.
- Input through the window's own dispatch (`BB_KEYS`): `cmd-shift-f`, the
  query typed key by key, `enter`, a pause, then a `state` step that prints
  the panel's hit count and the status line. Ground truth: `git grep` on the
  same tree.

## Result, after the speed change (`bench/BBEOD06/FIND.md`)

| Query | App | Ground truth |
|---|---|---|
| `ToolOutcome` (capital: case-sensitive) | 141 results in 1,236 files, 74 ms | 141 lines |
| `/Outcome$/` (regex; LF and CRLF files) | 29 results, 58 ms | 29 (`-P 'Outcome\r?$'`) |
| `locator`, then at once `ToolOutcome` | panel: 141 for `ToolOutcome` | the newer search wins |

First run, before the speed change: `ToolOutcome` 141 results, 245 ms;
`locator` 2,000 results "(stopped at 2000)", 7 ms (30,340 lines in all).

## Not covered

- A real click on a result row: the seat cannot post clicks (BB-TG-007).
  The window test `cmd_shift_f_searches_the_root_and_a_hit_opens_at_the_match`
  clicks through the same `jump` and checks file, line and column.
- Untracked files: the app searches 5 untracked, not ignored Apex files
  that `git grep` skips; none holds `ToolOutcome`, so the counts agree.
