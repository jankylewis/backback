# BB-CIP-004 — lesson

id: BB-CIP-004
family: CIP
status: FIXED
source: PR #88 follow-up (this entry)

## Who believed what

The merge job believed a grep over `gh pr checks` text proved all checks
green.

## What changed the belief

It let a red check through. It was a runner-capacity cancel, so no code was
at risk this time; the next red could be a real test failure.

## Rule left behind

Gate a merge on structured fields (`--json bucket`), all `pass`, never on
grepped text. A "not acquired by Runner" check is rerun, then merged.
