# BB-TG-015 readback: the runner flushes as a paint would (2026-11-03)

Four runs that ask no question, on a restored scratch copy of the
hand-check solution, MathOpsTests.cs opened from the command line: A
writable, `state` 2 s after Ready; B `--no-write`; C `--no-write`, a
`goto`, then `state` twice; D writable, `state` 6 s after Ready.

| Build | A | B | C | D |
|---|---|---|---|---|
| installed `e87dac2` (before) | `version=None` | `version=None` | `version=None` twice | `version=None` |
| branch `bb-tg-015-runner-flush` (after) | `version=Some(1)` | `version=Some(1)` | `version=Some(1)` twice | `version=Some(1)` |

`problems=0` in every run. Before, it read a tab the server never opened;
after, it is the server's own empty list for a file that builds clean.

## Verdict

PASS: a readback that asks no question now reads a list the server
computed. Runs that ask a question (every sweep) do not change: a paint or
a question had already sent everything, so the extra flush sends nothing.
