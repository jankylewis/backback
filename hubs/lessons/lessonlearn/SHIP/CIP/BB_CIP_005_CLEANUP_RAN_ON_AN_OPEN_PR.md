# BB-CIP-005 — lesson

id: BB-CIP-005
family: CIP
status: FIXED
source: PR #97

## Who believed what

The chain believed a successful `gh pr view` meant a merged PR.

## Rule left behind

Test the value, not the exit code, before any step that removes work;
and let git refuse (`-d`, no `--force`) as a second guard.
