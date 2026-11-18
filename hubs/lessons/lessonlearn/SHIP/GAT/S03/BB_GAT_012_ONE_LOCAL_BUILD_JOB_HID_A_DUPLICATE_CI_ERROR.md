# BB-GAT-012 — lesson

id: BB-GAT-012
family: GAT
status: OBSERVED
source: PR #116 CI, 2026-11-19

## Who believed what

The local gate saw one error, so cargo prints one error per mistake.

## What changed the belief

CI's parallel jobs compiled the lib and its test build both, and each
printed the same error.

## Rule left behind

Output read from a tool that runs units in parallel is checked with more
than one job before it is counted; the local one-job rule hides
what only parallel scheduling prints.
