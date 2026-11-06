# BB-CIP-004 — fix

id: BB-CIP-004
family: CIP
status: FIXED
source: PR #88 follow-up (this entry)

## The fix that holds

Merge only when `gh pr checks <pr> --json name,bucket` lists every check
with `bucket` `pass` and none `pending`, `fail`, `cancel` or `skipping`. A
check that no runner took is rerun (`gh run rerun <id>`), never merged
over. Wrong fix: grepping the text table.

## Proof

Rerun of run 42079599220 on `a164fa8`: `completed success`; PR #88 checks
now `gate pass`, `gate pass`.
