# BB-CIP-005 — fix

id: BB-CIP-005
family: CIP
status: FIXED
source: the ship steps (no repo code)

## The fix that holds

Cleanup is gated on the state's value, never on the command's exit:
`[ "$(gh pr view N --json state -q .state)" = MERGED ] && git worktree
remove <path> && git branch -d <branch>`. No `--force` on a worktree and
`-d`, not `-D`, on the branch: git itself then refuses unmerged work.

## Proof

PR #97: the worktree came back from `origin/perf-roslyn-direct` at the
pushed commit `daf397d`; the next ship ran the gated form.
