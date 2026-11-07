# BB-CIP-005 — Cleanup ran on an open PR

id: BB-CIP-005
family: CIP
status: FIXED
date: 2026-11-07
signature: the merge step printed `OPEN`, then the worktree and branch were removed
source: self-detected, PR #97 (CI red, not merged)
links: fix=yes lesson=yes evidence=yes toolgap=none

## What it is

The ship command was `guard && merge; gh pr view --json state && git
worktree remove --force && git branch -D`. With CI red the guard stopped
the merge, but `gh pr view` exits 0 for any state, so `OPEN` still ran the
cleanup. Nothing was lost: the commit was on `origin`; the worktree was
added back from it. The rule "remove a worktree only after MERGED" was
in force; the chain did not check it.
