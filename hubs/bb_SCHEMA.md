# hubs — schema and join key

Every hub of backback lives under `hubs/`. Today: the learnings hubs below.
Every bug, failure, fix, lesson and tool gap is recorded here.
Pattern taken from the NmcAx `hubs/` (one id, many hubs, build the path, never
search); ids, families and layers are backback's own.

## Hubs

| Hub | Path | Answers |
|---|---|---|
| `bugs/bugfacing/` | `S<nn>/BB_BUG_<nnn>_<SLUG>.md` | what the user faced: steps, expected, actual, status |
| `bugs/bugfix/` | `S<nn>/BB_BUG_<nnn>_<SLUG>.md` | the change, the PR, the red-first test, the proof |
| `failures/failurereason/` | `<LAYER>/<FAM>/BB_<FAM>_<nnn>_<SLUG>.md` | the mechanism, how it shows, how to confirm it |
| `failures/failfix/` | same path as failurereason | the fix that holds, the wrong fix, the proof |
| `failures/evidence/` | `bb_INDEX.md` rows (a file only past 3 paths) | PATHS to kept evidence, what to capture next time |
| `lessons/lessonlearn/` | same path as failurereason | who believed what, what changed it, the rule left |
| `lessons/toolgap/` | `BB_TG_<nnn>_<SLUG>.md` (shelved past 4) | missing or broken tool, workaround, status |

A bug is a symptom a user meets in the app (`BB-BUG-nnn`, global numbers).
A failure is a mechanism: a cause found in a bug, test, gate, CI run or
readback (`BB-<FAM>-nnn`). A bug links to its cause with `cause:`.
`TRIAGE.md` has one row per failure reason and per bug: read it first.

## Layers and families

| Layer | Code | Family |
|---|---|---|
| `APP` | `UIX` | window, menus, keys, Explorer, fonts, rendering |
| `APP` | `LSP` | Roslyn language server, diagnostics, completion |
| `APP` | `DAP` | netcoredbg, breakpoints, Set Next Statement |
| `APP` | `TST` | test engine, Test Explorer, runsettings |
| `SYS` | `PRC` | child processes, lifecycle, quit |
| `SYS` | `PRF` | speed, memory, typing latency |
| `SYS` | `ENV` | the machine: load, toolchain, permissions |
| `SHIP` | `BLD` | build, `.app` bundle, install, release |
| `SHIP` | `CIP` | GitHub CI differs from the local machine |
| `SHIP` | `GAT` | the gate, `bb audit`, tests that test the tests |

## Folders

Every folder holds one `bb_INDEX.md` (its `bb_` marker) and at most 4 entries
(HD law: 5 per folder). The hub's own `bb_INDEX.md` has the full table.
Shelves: `S01` = ids 001-004, `S02` = 005-008, and so on. Bugs are always
shelved; a family folder or `toolgap/` is shelved once it passes 4 ids.
Folders are made only when an entry needs them.

## Entry head

Title `# <id> — <short name>`, then one line each:

```
id: BB-UIX-001            (or BB-BUG-001, BB-TG-001)
family: UIX
status: OPEN | OBSERVED | FIXED
date: <first seen, YYYY-MM-DD>
signature: <exact text to grep in a new log; failurereason and bugfacing only>
source: <tracked path + line, commit, or PR>   (one per source)
observed: <what was seen outside the repo, sanitized, no personal paths>
links: fix=<yes|no> lesson=<yes|no> evidence=<yes|no> toolgap=<BB-TG-nnn|none>
cause: <BB-FAM-nnn>       (bugfacing only, when proven)
```

| Status | Means |
|---|---|
| `OPEN` | Cause not proven. Write "not proven" for each unproven part. |
| `OBSERVED` | Proven by a cited source; fix not in place on `master` or not proven. |
| `FIXED` | Fix on `master` AND a cited proof: red-first test then green, gate run, readback. |

## Laws

1. Every bug report, red gate, CI failure or false claim gets an entry in the
   same PR that fixes it; a fix PR without its entries is not done.
2. Never make an empty hub file to complete a set: a missing `failfix` file means
   no proven fix.
3. Every claim cites a source. No invented case, no guessed cause.
4. Public repo: no secrets, tokens, `.env` contents, e-mails, person names or
   absolute personal paths. Name people by role (`headmate`).
5. Ids are never reused. Check `TRIAGE.md` before giving a new id.
