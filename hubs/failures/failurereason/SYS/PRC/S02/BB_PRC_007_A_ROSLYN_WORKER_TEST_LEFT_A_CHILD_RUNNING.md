# BB-PRC-007 — A Roslyn worker test left a child running

id: BB-PRC-007
family: PRC
status: OBSERVED
date: 2026-11-06
signature: LEAK [ 11.252s] backback::bin/backback intel::bb_worker::tests::the_server_thread_counts_projects_then_syncs_edits_and_answers
source: self-detected in PR #93's CI run (cargo nextest, 10 workers); not in PR #92's run
links: fix=no lesson=no evidence=yes toolgap=none

## What it is

nextest marks a test "leaky" when a child it started still holds the
test's stdout or stderr after the test returns. This worker test starts
Roslyn (and dotnet build hosts); one of them outlived the test once. The
test passed, so `cargo test` never showed it.

## Next

Find which child holds the pipe (Roslyn, a `dotnet` build server, or
`VBCSCompiler`), and end it with the worker (`OwnedGroup`) or pass
`--disable-build-servers`. Until fixed, an app quit may also leave it.
